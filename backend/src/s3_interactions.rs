use rusoto_core::{credential::ProfileProvider, HttpClient, Region};
use rusoto_s3::{CreateBucketRequest, S3Client, S3};

// create the bucket we use if it doesn't exist yet
pub async fn create_bucket_if_needed(s3_loc: &str, bucket_name: &str) {
    let s = get_s3_client(s3_loc);

    match s.list_buckets().await {
        Err(e) => panic!("nooooo #{:?}", e),
        Ok(r) => {
            // check if our bucket is available
            info!("result is all #{:?}", r);
            if r.buckets.is_some() {
                match r
                    .buckets
                    .unwrap()
                    .iter()
                    .any(|x| x.name.as_ref().unwrap() == bucket_name)
                {
                    true => {
                        info!("bucket present, let's rock");
                        return;
                    }
                    false => info!("need to create bucket"),
                }
            }
            // if we got here it's time to create the bucket
            info!("create ze bucket");
            let cb_req = CreateBucketRequest {
                bucket: bucket_name.to_string(),
                ..Default::default()
            };
            let cb_res = s.create_bucket(cb_req).await;
            match cb_res {
                Err(e) => panic!("couldn't create bucket when it didn't exist: #{:?}", e),
                Ok(o) => info!("created bucket: #{:?}", o),
            }
        }
    }
}

// handle local vs real S3
fn get_s3_client(s3_loc: &str) -> S3Client {
    // be nice to not have to do this all the time. Use lazy_static?
    match s3_loc.replace("\n", "").len() {
        0 => {
            info!("Using real S3 with a new client");
            // use profile provider only
            let profile_creds =
                ProfileProvider::new().expect("Couldn't make new Profile credential provider");
            let http_client = HttpClient::new().expect("Couldn't make new HTTP client");
            S3Client::new_with(http_client, profile_creds, Region::UsWest2)
        }
        _ => {
            info!("Using local S3 with a new client");
            S3Client::new(Region::Custom {
                name: "us-east-1".into(), // local testing only
                endpoint: s3_loc.into(),
            })
        }
    }
}
