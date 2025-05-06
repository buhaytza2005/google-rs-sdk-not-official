use anyhow::Result;
use google_api_rust_client_not_official::{
    auth::service_account::ServiceAccountCredentials,
    services::{
        business_service::{
            locations::{Location, Locations},
            BusinessRequest, BusinessService,
        },
        ServiceBase,
    },
};
use std::{path::PathBuf, str::FromStr};

static MY_BUSINESS_SERVICE_SCOPE: &str = "https://www.googleapis.com/auth/plus.business.manage";

#[tokio::main]
async fn main() -> Result<()> {
    let locations = fn_get_locations_details().await?;

    //let locations_with_details = fn_get_locations_details().await?;
    //println!("{:#?}", locations_with_details);

    // let _ = fn_update_location().await?;
    //let _ = fn_review_by_location().await?;
    //let _ = get_location_details().await?;
    //
    //
    let to_update = locations
        .locations
        .iter()
        .filter(|l| {
            l.categories.is_some()
                && l.categories.clone().unwrap().primary_category.unwrap().name
                    != "categories/gcid:car_wash"
        })
        .collect::<Vec<_>>();
    let wsm = locations.locations.iter().find(|l| l.store_code == "5331");
    let updates = locations
        .locations
        .iter()
        .filter(|l| !l.title.contains("Waves Hand Car Wash") || l.title.len() < 17)
        .collect::<Vec<_>>();
    println!("{:#?}", wsm);
    println!("{:#?}", updates);

    Ok(())
}

async fn get_location_details() -> Result<()> {
    let access_token = get_token().await;
    let mut business_service = BusinessService::new(&access_token);

    let location_id = "locations/559469233876173390";
    let read_mask = vec!["openInfo", "storeCode", "title", "name", "phoneNumbers"];
    let loc = business_service
        .get_location_details(location_id, read_mask)
        .await
        .expect("should have a location with the details");

    println!("{:#?}", loc);
    Ok(())
}
async fn get_token() -> String {
    let filepath: PathBuf = PathBuf::from_str("credentials.json").expect("Is this missing?");
    let credentials = ServiceAccountCredentials::from_service_account_file(filepath)
        .expect("this other one missing?");

    let creds = ServiceBase::new_with_credentials(credentials, vec![MY_BUSINESS_SERVICE_SCOPE]);
    let access_token = creds
        .service_account_credentials
        .expect("should have creds")
        .get_access_token()
        .await
        .expect("Should have token");

    access_token
}
async fn fn_get_locations() -> Result<Locations> {
    let access_token = get_token().await;
    let mut business_service = BusinessService::new(&access_token);

    let locations = business_service.get_locations("-").await?;

    println!("got {} locations", locations.locations.len());
    Ok(locations)
}
async fn fn_get_locations_details() -> Result<Locations> {
    let access_token = get_token().await;
    let mut business_service = BusinessService::new(&access_token);
    let mask = vec!["name", "storeCode", "title", "metadata", "categories"];

    let locations = business_service.get_locations_details("-", mask).await?;

    println!("got {} locations", locations.locations.len());
    Ok(locations)
}

async fn fn_update_location() -> Result<()> {
    let mut loc = Location::default();

    let access_token = get_token().await;
    let mut business_service = BusinessService::new(&access_token);
    let _ = business_service
        .update_location(&loc, vec!["title"])
        .await?;

    Ok(())
}

async fn fn_review_by_location() -> Result<()> {
    let access_token = get_token().await;
    let mut business_service = BusinessService::new(&access_token);

    //let locations = business_service.get_locations("-").await?;
    let loc = Location {
        name: "locations/5031657144081502405".to_string(),
        title: "Waves Hand Car Wash Sutton-Cheam".to_string(),
        store_code: "3235".to_string(),
        ..Default::default()
    };
    let revs = business_service.reviews_by_location(&loc, None).await?;

    println!("{revs:#?}");

    println!("{:#?}", revs.len());
    /*
    for loca in locations.locations {
        business_service.review_summary(&loca).await?;
    }
    */

    Ok(())
}
