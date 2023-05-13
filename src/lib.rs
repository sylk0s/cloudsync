//! # Cloudsync
//! `cloudsync` provides a trait which allows serializable objects to be easily saved to a firestore database
//! 
//! ## Setup
//! - go to firebase console for your google account
//! - click add project & setup a new project
//! - when the project opens, in the bar on the left, click on settings next to project overview
//! - click on service accounts, then generate new private key. The JSON this downloads is the credential file.
//! - move this file somewhere safe (for testing, I put in the project root under the name firebase.json)
//!
//! ## Usage
//! - Make sure the object you want to extend satisfies the trait bounds (notably Serialize and Deserialize)
//! - impl Unique and CloudSync for the object (you should just need to implement `uuid()` and `config()`)
//! - If you set everything up correctly, it should work!

use firestore::{FirestoreDb, FirestoreQueryParams, FirestoreDbOptions, FirestoreQueryCollection};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use gcloud_sdk::TokenSourceType;
use std::path::PathBuf;

/// Internal error type
type Error = Box<dyn std::error::Error + Send + Sync>;

/// Get the correct FireStore database object with the specified configs and credentials
async fn get_fs_db(cfg: &CLConfig) -> Result<FirestoreDb, Error> {
    Ok(FirestoreDb::with_options_token_source(
        FirestoreDbOptions::new(cfg.project_id.clone(),),
        gcloud_sdk::GCP_DEFAULT_SCOPES.clone(),
        TokenSourceType::File(PathBuf::from(&cfg.cred_path)),
    ).await?)
}


/// Allows a serializable object to be saved in the cloud using firestore
#[async_trait]
pub trait CloudSync<T> where 
    for<'a> Self: Deserialize<'a> + Serialize + Unique<T> + Sync + Send,
    T: Serialize + std::fmt::Display + std::cmp::Eq + std::hash::Hash + Send + Sync {

    // Save an object to the collection specified in the config
    async fn save(&self) -> Result<(), Error> {
        let cfg = Self::config();
        let db = get_fs_db(&cfg).await?;
        db.delete_by_id(&cfg.collection, self.uuid().to_string()).await?;
        db.create_obj(&cfg.collection, self.uuid().to_string(), self).await?;
        Ok(())
    }

    /// Remove this object from the collection
    async fn rm(&self) -> Result<(), Error> {
        let cfg = Self::config();
        let db = get_fs_db(&cfg).await?;
        db.delete_by_id(&cfg.collection, self.uuid().to_string()).await?;
        Ok(())
    }

    /// Get all objects from a collection in a vector
    /// This is the typical manner in which you would iterate over all of the objects in the same collection as this one
    async fn get() ->  Result<Vec<Self>, Error> {
        let cfg = Self::config();
        let db = get_fs_db(&cfg).await?;
        let objects: Vec<Self> = db.query_obj(FirestoreQueryParams::new(FirestoreQueryCollection::Single(cfg.collection))).await?;
        Ok(objects)
    }

    /// Get all items from the collection this object is in as a HashMap
    /// This is the typical manner in which you would find a specific object
    async fn hash() -> Result<HashMap<T, Self>, Error> {
        let cfg = Self::config();
        let db = get_fs_db(&cfg).await?;
        let objects: Vec<Self> = db.query_obj(FirestoreQueryParams::new(FirestoreQueryCollection::Single(cfg.collection))).await?;
        let mut hash = HashMap::new();
        for obj in objects {
            hash.insert(obj.uuid(), obj);
        }
        Ok(hash)
    }

    // TODO
    // async fn this()
    
    /// Get this objects cloud config, not intended for use outside of the crate 
    fn config() -> CLConfig;
}

/// Each object implementing this trait can provide a uuid for itself
pub trait Unique<T> where T: Serialize {

    /// Get the uuid of this object
    fn uuid(&self) -> T;
}

/// The config for how this object syncs with the cloud
/// 
/// # Fields:
/// - project_id: name of the the project in firebase
/// - cred_path: the location of the credentials json file downloaded from firebase
/// - collection: the name of the collection that objects of this type should be saved to
/// (note: you could write this code such that the collection changes based on paramteres in the object, this is untested)
///
pub struct CLConfig {
    pub project_id: String,
    pub cred_path: String,
    pub collection: String,
}

// Note: This testing setup just wont work unless you set everything up in firebase the exact same
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Serialize)]
    struct TestOBJ {
        key: String,
        data: String,
    }

    impl CloudSync<String> for TestOBJ {
        fn config() -> CLConfig {
            CLConfig {
                project_id: "cloudsync-testing".to_string(),
                cred_path: "./firebase.json".to_string(),
                collection: "testing".to_string(),
            }
        }
    }

    impl Unique<String> for TestOBJ {
        fn uuid(&self) -> String {
            return String::from(&self.key);
        }
    }

    // Super basic test...
    // Add more at a later time?
    #[tokio::test]
    async fn testSavingObject() {
        let obj = TestOBJ {
            key: "aaa".to_string(),
            data: "data".to_string(),
        };
        obj.save().await;
        let vec = TestOBJ::get().await.unwrap();
        assert_eq!(vec.len(), 1);
    }
}