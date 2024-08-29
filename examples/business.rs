pub mod locations;

use anyhow::Result;
use google_api_rust_client_not_official::{
    auth::service_account::ServiceAccountCredentials,
    services::{
        business_service::{BusinessRequest, BusinessService},
        ServiceBase,
    },
};
use std::{path::PathBuf, str::FromStr};
static MY_BUSINESS_SERVICE_SCOPE: &str = "https://www.googleapis.com/auth/plus.business.manage";

#[tokio::main]
async fn main() -> Result<()> {
    let filepath: PathBuf = PathBuf::from_str("credentials.json").expect("Is this missing?");
    let credentials = ServiceAccountCredentials::from_service_account_file(filepath)
        .expect("this other one missing?");

    let creds = ServiceBase::new_with_credentials(credentials, vec![MY_BUSINESS_SERVICE_SCOPE]);
    let access_token = creds
        .service_account_credentials
        .expect("should have creds")
        .get_access_token()
        .await?;
    let mut business_service = BusinessService::new(&access_token);
    let accounts = business_service.accounts().await?;

    let my_account = &accounts.accounts.unwrap()[0];
    let acc_id = my_account.name.split("/").collect::<Vec<_>>()[1].to_string();

    let locations = business_service.get_locations("-").await?;

    let admins = business_service
        .admins(locations.locations.as_ref())
        .await?;
    println!("{:#?}", admins);

    /*
    for location in &locations.locations {
        let reviews = business_service.review_summary(location).await?;
    }
    */
    Ok(())
}
