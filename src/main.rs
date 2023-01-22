use std::{fs, thread};
use std::path::Path;
use std::io::{Write, ErrorKind};
use std::time::Duration;
use chrono::prelude::*;
use select::document::Document;
use select::predicate;

fn main() {
    // Make sure the universe hasn't collapsed
    println!("Hello, world!");

    // Instantiate the client and note the domain's url
    let client = reqwest::Client::new();
    let domain_url = "https://twokinds.keenspot.com";

    // Create our output directory
    let root_dir: String = "download".to_owned();
    match fs::create_dir(&root_dir) {
        Ok(dir) => dir,
        Err(error) => match error.kind() {
            // We can proceed if the directory already exists
            ErrorKind::AlreadyExists => (),
            // However, if the directory truly doesn't exist, then panic.
            _ => panic!("Could not create root directory.")
        }
    }

    // Create an HTML file to display all the images
    let mut index = fs::File::create(root_dir.clone() + "/index.html")
        .expect("Could not open file to save image");
        
    let date = Utc::now().format("%Y-%m-%d").to_string();

    index.write_all(format!(
"<html>
<head>
  <title>TwoKinds</title>
  <style>
    html {{ background: black; }}
    * {{ font-family: monospace; }}
    h1, label {{ color: white; }}
    input {{ width: 5em; }}
    img {{
      width: 100%;
      height: auto;
      margin-bottom: 5px;
    }}
  </style>
</head>
<script>
window.onload = function() {{
  // Add an event listener to the form so we can cancel it and handle it ourselves
  let form = document.getElementById('page-form');
  form.addEventListener('submit', jump, false);
}}

function jump(event=null) {{
  // If we intercepted a form submit event, cancel it.
  if (event) event.preventDefault();

  // Get the value currently in the input box and jump to that image
  let page = document.getElementById('page-entry').value;
  location.hash = '#'+page
}}
</script>
<body>
  <h1>TwoKinds Complete Gallery ({})</h1>
  <form id='page-form'>
    <table>
      <tr>
        <td>
          <label for='page-entry'>Enter page number:</label>
          <input id='page-entry' type='number' min='1' max='1300'>
          <button onclick='jump()' type='button'>Jump</button>
        </td>
      </tr>
    </table>
  </form>\n\n",date)
    .as_bytes())
    .expect("Unable to write index header");

    // Fetch the archive index
    let mut url: String = domain_url.to_owned();
    url.push_str("/archive");
    let archive_resp = get_page(&client, &url);
    println!("Response status {}", archive_resp.status());

    // Parse the HTML into a searchable format
    let archive = Document::from_read(archive_resp)
      .expect("Error parsing archive document");

    // Find all the chapter sections and iterate over them
    for chapter in archive.find(predicate::Class("chapter")) {
      // Retrieve the unique identifer string for each chapter
      let chapter_id = chapter.attr("data-ch-id")
        .expect("Chapter does not have an id?");
      println!("chapter {:?}", chapter_id); // Print the id for progress tracking
      
      // Create the chapter directory
      let chapter_dir = root_dir.clone() + "/" + chapter_id;
      match fs::create_dir(&chapter_dir) {
        Ok(dir) => dir,
        Err(error) => match error.kind() {
          // We can proceed if the directory already exists
          ErrorKind::AlreadyExists => (),
          // However, if the directory truly doesn't exist, then panic.
          _ => panic!("Could not create chapter directory '{}'", chapter_dir)
        }
      }

      /*
        The following code traverses the structure of a "chapter" section and picks out information of interest.
        We're interested in the page link and the unique incremental page number.
        Below is a simplified example of a section, where n is the page number.

        <section class="chapter" data-ch-id="x">
          <h2>...</h2>
          <p>...</p>
        
          <aside>
            <a href="/comic/n">
              <span>n</span>
              <img ... >  // We don't care about this image because it's a thumbnail
            </a>
            ... many <a> tags mixed with <noscript>'s ...
          </aside>
        </section>
      */

      let aside = chapter.find(predicate::Name("aside"))
        .next()
        .expect("Chapter does not have an <aside> tag");

      for link in aside.find(predicate::Name("a")) {
        // Extract the href within the <a>
        let page_href = link.attr("href").unwrap();

        // Extract the text from the <span>
        let page_id = link.find(predicate::Name("span"))
          .next()
          .unwrap()
          .text();
        
        // Append the image to the HTML index
        let page_partialpath = chapter_id.to_owned() + "/" + &page_id + ".jpg";
        index.write_all(
          format!("  <img id='{page_id}' src='{page_partialpath}'>\n")
            .as_bytes())
          .expect("Could not append reference to index");
          
        // Don't attempt to download files we already have
        let page_fullpath = chapter_dir.clone() + "/" + &page_id + ".jpg";
        if Path::new(&page_fullpath).exists() {
          //println!("Skipping {page_id}");
          continue;
        }

        println!("page {page_id}: {page_href}");

        // Now we need to fetch the page via the href and parse it
        url = domain_url.to_owned();
        url.push_str(page_href);
        let page_resp = get_page(&client, &url);
        thread::sleep(Duration::from_secs(2));

        let page = Document::from_read(page_resp)
          .expect("Error parsing page document");

        /*
            A comic page contains an article tag, which follows a fixed structure.
            Like before, we pull out the image src, except this time we save it into the chapter folder as "page_id.jpg"
            Below is an example of an article on these pages for reference.

            <article class="comic">
              <header><h1>...</h1></header>
              <img src="/comics/YYYYMMDD.jpg">
            </article>
        */

        let article = page.find(predicate::Name("article"))
          .next()
          .expect("Page does not have an <article> tag");

        let page_imgsrc = article.find(predicate::Name("img"))
          .next()
          .unwrap()
          // Extract the src within the <img>
          .attr("src")
          .unwrap();
        
        // Download the image
        let mut page_img = get_page(&client, &page_imgsrc);
        //println!("{:?}", page_imgsrc);

        // Save it into the correct directory with its proper id
        let mut file = fs::File::create(&page_fullpath)
          .expect("Could not open file to save image");
        page_img.copy_to(&mut file)
          .expect("Unable to write image data. Out of space?");
    }
  }
  // Finalise the index
  index.write_all("</body>\n</html>".as_bytes())
    .expect("Unable to finalise index");
}

fn get_page(client: &reqwest::Client, url: &str) -> reqwest::Response {
  // Get the archive page
  let result = client
    .get(url)
    .send();
  
  // Return the response
  match result {
    Ok(response) => response,
    Err(error) => panic!("Error retrieving {:?}\nReponse status {:?}\n{:?}", url, error.status(), error),
  }
}
