use std::fs;
use std::path::{Path,PathBuf};
use std::io::ErrorKind;
use reqwest::{Client, Response};

#[tokio::main]
async fn main() {
  // Instantiate the client and note the domain's url
  let client = Client::new();
  let domain_url = "https://twokinds.keenspot.com";

  // Create our output directory
  let root_dir = Path::new("./download");
  match fs::create_dir(&root_dir) {
      Ok(dir) => dir,
      Err(error) => match error.kind() {
          // We can proceed if the directory already exists
          ErrorKind::AlreadyExists => (),
          // However if we truly couldn't create it, then panic.
          _ => panic!("Could not create root directory")
      }
  }

  // Fetch the archive page for information about all chapters and their pages
  let mut url = PathBuf::from(domain_url);
  url.push("archive");
  let response = get_page(&client, url).await;
  println!("{:?}", &response);
  let content = match response.text().await {
    Ok(content) => content,
    Err(why) => panic!("Failed to read response from /archive request! {why}")
  };
}

async fn get_page(client: &Client, url: PathBuf) -> Response {  
  // TODO: Consider using the retry crate here to handle rate limiting properly
  let path = url.to_str().unwrap();
  // Get the archive page
  let result = client.get(path)
    .send().await;

  // Return the response
  match result {
    Ok(response) => response,
    Err(error) => // This catches network errors but not HTTP errors
      panic!("Error retrieving {:?}\nReponse status {:?}\n{:?}",
             path, error.status(), error)
  }
}