use firestore::{FirestoreDb, FirestoreQueryParams};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/* TODO:
    - Configurable file rather than the .env / firebase.json
    - Docs for everything here
    - More informative error handling?
 */

type Error = Box<dyn std::error::Error + Send + Sync>;
const token_path: &str = "./firebase.json";

// TODO change this to not use .env
fn config_env_var(name: &str) -> Result<String, String> {
   std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

// Gets a firestore db without needing any env vars as is the default behavior
// Change this later so that project_id isn't needed anymore either
async fn get_fs_db() -> Result<FirestoreDb, Error> {
    FirestoreDb::with_options_token_source(
        FirestoreDbOptions::new(
            &config_env_var("PROJECT_ID")?,
        ),
        gcloud_sdk::GCP_DEFAULT::SCOPES.clone(),
        TokenSourceType::Json(token_path),
    ).await?
}

#[async_trait]
// Can sync with firebase
pub trait CloudSync where for<'a> Self: Deserialize<'a> + Serialize + Unique + Sync + Send {

    // Save an object [obj] to a specific collection
    async fn save(&self, collection: &'static str) -> Result<(), Error> {
        let db = get_fs_db().await?;
        db.delete_by_id(collection, self.uuid().to_string()).await?;
        db.create_obj(collection, self.uuid().to_string(), self).await?;
        Ok(())
    }

    // Remove a specific object
    async fn rm(&self) -> Result<(), Error> {
        let db = get_fs_db().await?;
        db.delete_by_id(Self::clname(), self.uuid().to_string()).await?;
        Ok(())
    }

    /// Get all objects from a collection
    async fn get() ->  Result<Vec<Self>, Error> {
        let db = get_fs_db().await?;
        let objects: Vec<Self> = db.query_obj(FirestoreQueryParams::new(Self::clname().into())).await?;
        Ok(objects)
    }

    /// Get all items from the collection as a HashMap
    async fn hash() -> Result<HashMap<u64, Self>, Error> {
        let db = get_fs_db().await?;
        let objects: Vec<Self> = db.query_obj(FirestoreQueryParams::new(Self::clname().into())).await?;
        let mut hash = HashMap::new();
        for obj in objects {
            hash.insert(obj.uuid(), obj);
        }
        Ok(hash)
    }

    // TODO implement a wrapper so that you only get one specific object via uuid?

    // Get the name associated with a type implemeneting this trait.
    fn name() -> &'static str;
}

// Ensures the object can provide a unique id
pub trait Unique {
    fn uuid(&self) -> u64;
}