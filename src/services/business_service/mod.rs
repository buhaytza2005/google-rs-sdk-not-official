pub mod accounts;
pub mod endpoint;
pub mod locations;

use accounts::{Accounts, Admins, PageAdmins};
use anyhow::{anyhow, Result};
use endpoint::EndPoint;
use futures::stream::{FuturesUnordered, StreamExt};
use locations::{Location, Locations};
use log::info;
use reqwest::{
    header::{self, HeaderValue},
    Response,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct BusinessService {
    access_token: String,
    account_id: Option<String>,
}

pub trait BusinessRequest {
    fn request(
        &mut self,
        endpoint: EndPoint,
    ) -> impl std::future::Future<Output = Result<Response>> + Send;

    fn resource_request(
        &mut self,
        endpoint: EndPoint,
        next_page_token: Option<serde_json::Value>,
    ) -> impl std::future::Future<Output = Result<Response>> + Send;
    fn update_request(
        &mut self,
        endpoint: EndPoint,
        payload: &Location,
    ) -> impl std::future::Future<Output = Result<Response>> + Send;

    fn accounts(&mut self) -> impl std::future::Future<Output = Result<Accounts>> + Send;

    fn get_locations(
        &mut self,
        account_id: &str,
    ) -> impl std::future::Future<Output = Result<Locations>> + Send;

    fn get_locations_details<T: Into<String> + Send>(
        &mut self,
        account_id: &str,
        read_mask: Vec<T>,
    ) -> impl std::future::Future<Output = Result<Locations>> + Send;

    fn update_location(
        &mut self,
        location: &Location,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn admin(
        &mut self,
        location: &Location,
    ) -> impl std::future::Future<Output = Result<PageAdmins>> + Send;

    fn admins(
        &mut self,
        location: &Vec<Location>,
    ) -> impl std::future::Future<Output = Result<Vec<PageAdmins>>> + Send;

    fn reviews_by_location(
        &mut self,
        location: &Location,
    ) -> impl std::future::Future<Output = Result<Value>> + Send;

    fn review_summary(
        &mut self,
        location: &Location,
    ) -> impl std::future::Future<Output = Result<Value>> + Send;
}

impl BusinessService {
    pub fn new(access_token: &str) -> Self {
        BusinessService {
            access_token: access_token.to_string(),
            ..Default::default()
        }
    }
}

impl BusinessRequest for BusinessService {
    async fn request(&mut self, endpoint: EndPoint) -> Result<Response> {
        let url = EndPoint::build(endpoint).expect("could not build accounts url");

        let client = reqwest::Client::builder().build()?;
        let res = client
            .get(url)
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.access_token.as_str())).unwrap(),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("Error with request");

        Ok(res)
    }
    async fn resource_request(
        &mut self,
        endpoint: EndPoint,
        next_page_token: Option<serde_json::Value>,
    ) -> Result<Response> {
        let mut url = EndPoint::build(endpoint).expect("could not build accounts url");
        if let Some(token) = next_page_token {
            url.push_str(format!("&pageToken={}", token.as_str().unwrap()).as_str())
        }

        let client = reqwest::Client::builder().build()?;
        let res = client
            .get(url)
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.access_token.as_str())).unwrap(),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("Error with request");

        Ok(res)
    }

    async fn update_request(&mut self, endpoint: EndPoint, payload: &Location) -> Result<Response> {
        let mut url = EndPoint::build(endpoint).expect("could not build accounts url");
        url.push_str("?updateMask=title");
        let client = reqwest::Client::builder().build()?;
        let res = client
            .patch(url)
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.access_token.as_str())).unwrap(),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(payload)
            .send()
            .await
            .expect("Error with patch request");

        Ok(res)
    }
    async fn accounts(&mut self) -> Result<Accounts> {
        let response = self.request(EndPoint::AccountsEndpoint).await?;
        let accounts: Accounts = response.json().await?;
        if accounts.accounts.len() == 0 {
            return Err(anyhow!("no accounts, something went wrong!"));
        }
        Ok(accounts)
    }
    /// must be sequential as the `nextPageToken` is needed to process the rest of the locations
    ///
    /// * `account id` - ID of account that manages the locations, for service account use `"-"`
    async fn get_locations(&mut self, account_id: &str) -> Result<Locations> {
        let mut locations = Locations::default();
        let mut next_page_token = None;
        loop {
            let response = self
                .resource_request(
                    EndPoint::LocationsEndpoint(account_id.into()),
                    next_page_token.clone(),
                )
                .await?;
            let resp: Value = response.json().await?;
            let val_pages = &resp.get("locations").unwrap().as_array().unwrap().clone();
            let pages: Vec<Location> = val_pages
                .iter()
                .map(|v| serde_json::from_value(v.clone()).unwrap())
                .collect();
            locations.locations.extend(pages);
            next_page_token = resp.get("nextPageToken").cloned();
            if next_page_token.is_none() {
                break;
            };
        }
        info!("Retrieved {} locations", locations.locations.len());
        Ok(locations)
    }
    /// must be sequential as the `nextPageToken` is needed to process the rest of the locations
    ///
    ///```rust
    ///let mask = vec![
    ///     "storeCode",
    ///     "title",
    ///     "name",
    ///     "phoneNumbers"
    ///];
    ///let access_token = get_token().await;
    ///let mut business_service = BusinessService::new(&access_token);

    ///let locations = business_service.get_locations_details("-", mask).await?;
    ///
    ///```
    ///
    /// * `account_id` - account that manages the location, for sys users, use `"-"`
    /// * `read_mask` - Vector of String or &str with the parts of the mask https://developers.google.com/my-business/reference/businessinformation/rest/v1/accounts.locations#Location
    async fn get_locations_details<T: Into<String> + Send>(
        &mut self,
        account_id: &str,
        read_mask: Vec<T>,
    ) -> Result<Locations> {
        let mut locations = Locations::default();
        let mut next_page_token = None;
        let read_mask_str: Vec<String> = read_mask.into_iter().map(Into::into).collect();
        let read_mask_joined = read_mask_str.join(",");
        loop {
            let response = self
                .resource_request(
                    EndPoint::LocationsDetailsEndpoint(account_id.into(), read_mask_joined.clone()),
                    next_page_token.clone(),
                )
                .await?;
            let resp: Value = response.json().await?;
            let val_pages = &resp.get("locations").unwrap().as_array().unwrap().clone();
            let pages: Vec<Location> = val_pages
                .iter()
                .map(|v| serde_json::from_value(v.clone()).unwrap())
                .collect();
            locations.locations.extend(pages);
            next_page_token = resp.get("nextPageToken").cloned();
            if next_page_token.is_none() {
                break;
            };
        }
        info!("Retrieved {} locations", locations.locations.len());
        Ok(locations)
    }

    async fn admin(&mut self, location: &Location) -> Result<PageAdmins> {
        let endpoint = EndPoint::AdminEndpoint(location.name.clone());

        let response = self.request(endpoint).await?;
        let resp: Admins = response.json().await?;

        Ok(PageAdmins {
            page_name: location.name.clone(),
            page_title: location.title.clone(),
            store_code: location.store_code.clone(),
            admin_count: resp.admins.len(),
            admins: resp.admins,
        })
    }

    async fn admins(&mut self, locations: &Vec<Location>) -> Result<Vec<PageAdmins>> {
        let mut futures = FuturesUnordered::new();
        let mut results: Vec<PageAdmins> = Vec::new();

        for location in locations {
            let mut self_clone = self.clone();
            futures.push(async move { self_clone.admin(location).await })
        }

        while let Some(result) = futures.next().await {
            match result {
                Ok(admin) => results.push(admin),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }

    async fn reviews_by_location(&mut self, location: &Location) -> Result<Value> {
        let endpoint = EndPoint::Reviews("-".to_string(), location.name.clone());
        let res = self.request(endpoint).await.expect("should have reviews");

        let resp: serde_json::Value = res.json().await.expect("should have json");
        println!("{:#?}", resp);
        Ok(resp)
    }

    async fn review_summary(&mut self, location: &Location) -> Result<Value> {
        let endpoint = EndPoint::Reviews("-".to_string(), location.name.clone());
        let res = self
            .request(endpoint)
            .await
            .expect("should have reviews for site");

        if !res.status().is_success() {
            println!("{:#?}", res.status());
        }

        let resp: serde_json::Value = res.json().await.expect("should have json");
        let total_reviews = resp.get("totalReviewCount").unwrap_or(&Value::Null);
        let rating = resp.get("averageRating").unwrap_or(&Value::Null);
        println!("{:#?}", location);
        //println!("{:#?}", resp);
        println!(
            "{:#?} - total reviews {} - average rating {}",
            location.title, total_reviews, rating
        );
        Ok(resp)
    }

    async fn update_location(&mut self, location: &Location) -> Result<()> {
        let endpoint = EndPoint::Location(location.name.clone());

        let res = self
            .update_request(endpoint, location)
            .await
            .expect("Should update");

        let resp: Location = res.json().await?;
        println!("{:#?}", resp);

        Ok(())
    }
}
