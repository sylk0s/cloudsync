# Cloudsync
`cloudsync` provides a trait which allows serializable objects to be easily saved to a firestore database

## Setup
- go to firebase console for your google account
- click add project & setup a new project
- when the project opens, in the bar on the left, click on settings next to project overview
- click on service accounts, then generate new private key. The JSON this downloads is the credential file.
- move this file somewhere safe (for testing, I put in the project root under the name firebase.json)
 
## Usage
- Make sure the object you want to extend satisfies the trait bounds (notably Serialize and Deserialize)
- impl Unique and CloudSync for the object (you should just need to implement `uuid()` and `config()`)
- If you set everything up correctly, it should work!
