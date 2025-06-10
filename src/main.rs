#![allow(dead_code)]

use std::fmt::Write;
use std::time::Duration;
use std::{collections::HashSet, error::Error};

use convert_case::{Case, Casing};
use headless_chrome::Browser;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
	for id in 1..=500
	{
		let contents = get_feat_contents(id)?;
		let html = Html::parse_fragment(&contents);
		if is_valid_page(&html)?
		{
			let name = extract_feat_name(&html)?.to_case(Case::Snake);
			tokio::spawn(async move { write(format!("./feats/{name}.html"), contents).await });
		}
		else
		{
			println!("Found invalid feat at id {id}!");
		}

		tokio::time::sleep(Duration::new(1, 0)).await;
	}
	Ok(())
}

async fn write(file_name: String, contents: String) -> Result<(), Box<dyn Error + Sync + Send>>
{
	tokio::fs::write(file_name, contents).await?;
	Ok(())
}

fn get_feat_contents(id: u16) -> Result<String, Box<dyn Error>>
{
	let browser = Browser::default()?;
	let tab = browser.new_tab()?;
	tab.navigate_to(&format!("https://2e.aonprd.com/Feats.aspx?ID={id}"))?;
	println!("Waiting for main content of feat {id}");
	if let Ok(main_content) =
		tab.wait_for_element("#ctl00_RadDrawer1_Content_MainContent_DetailedOutput")
	{
		Ok(main_content.get_content()?)
	}
	else
	{
		println!("Couldn't find main content, waiting for body...");
		Ok(tab.wait_for_element("body")?.get_content()?)
	}
}

fn is_valid_page(html: &Html) -> Result<bool, Box<dyn Error>>
{
	Ok(!html
		.select(&Selector::parse("span > h1")?)
		.any(|elem| elem.inner_html().contains("Server Error in")))
}

fn extract_feat_name(html: &Html) -> Result<String, Box<dyn Error>>
{
	let element = html
		.select(&Selector::parse("h1 > span + a")?)
		.next()
		.expect("oops!");
	Ok(element.inner_html())
}
