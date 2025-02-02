#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use alloc::string::ToString;

const WWW_URL: &str = "https://www.mxshm.site";

const FILTER_TAG: [&str; 1] = ["全部"];
const FILTER_AREA: [&str; 3] = ["-1", "1", "2"];
const FILTER_END: [&str; 3] = ["-1", "0", "1"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut tag = String::new();
	let mut area = String::new();
	let mut end = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"题材" => {
						tag = FILTER_TAG[index].to_string();
					}
					"地区" => {
						area = FILTER_AREA[index].to_string();
					}
					"进度" => {
						end = FILTER_END[index].to_string();
					}
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let url = if query.is_empty() {
		format!(
			"{}/booklist?tag={}&area={}&end={}&page={}",
			WWW_URL,
			encode_uri(tag),
			area,
			end,
			page
		)
	} else {
		format!("{}/search?keyword={}", WWW_URL, encode_uri(query.clone()))
	};
	let html = Request::new(url, HttpMethod::Get).html()?;
	let has_more = query.is_empty();
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".mh-item").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select("a")
			.attr("href")
			.read()
			.split("/")
			.map(|a| a.to_string())
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let cover = item
			.select("a>p")
			.attr("style")
			.read()
			.replace("background-image: url(", "")
			.replace(")", "");
		let title = item
			.select(".mh-item-detali>h2>a")
			.text()
			.read()
			.trim()
			.to_string();
		mangas.push(Manga {
			id,
			cover,
			title,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/book/{}", WWW_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let cover = html
		.select(".banner_detail_form>.cover>img")
		.attr("src")
		.read();
	let title = html.select(".info>h1").text().read().trim().to_string();
	let author = html
		.select(".info>p:nth-child(4)")
		.text()
		.read()
		.trim()
		.replace("作者：", "")
		.split("&")
		.map(|a| a.to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	let description = html
		.select(".info>.content>span>span")
		.text()
		.read()
		.trim()
		.to_string();
	let categories = html
		.select(".info>p:nth-child(6)>span>a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = match html
		.select(".info>p:nth-child(5)>span>span")
		.text()
		.read()
		.trim()
		.to_string()
		.as_str()
	{
		"连载中" => MangaStatus::Ongoing,
		"已完结" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};
	let nsfw = MangaContentRating::Nsfw;
	let viewer = MangaViewer::Scroll;

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/book/{}", WWW_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#detail-list-select>li>a").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.attr("href")
			.read()
			.split("/")
			.map(|a| a.to_string())
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let title = item.text().read().trim().to_string();
		let chapter = (index + 1) as f32;
		let url = format!("{}/chapter/{}", WWW_URL, id.clone());
		chapters.push(Chapter {
			id,
			title,
			chapter,
			url,
			..Default::default()
		});
	}
	chapters.reverse();

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/chapter/{}", WWW_URL, chapter_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select(".comicpage>div>img").array().enumerate() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let index = index as i32;
		let url = item.attr("data-original").read().trim().to_string();
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}
