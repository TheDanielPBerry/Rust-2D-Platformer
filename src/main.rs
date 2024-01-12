

use std::collections::HashMap;

use macroquad::prelude::*;

mod article;
mod world;
use article::article::Article;
use crate::world::world::*;


fn window_conf() -> Conf {
	Conf {
		window_title: "Game".to_owned(),
		fullscreen: false,
		..Default::default()
	}
}


fn get_camera_track(camera_track: Vec2, camera: &mut Article) -> Vec2 {
	
	if is_key_down(KeyCode::F) {
		print!("a");
	}
	let camera_track_bounds = vec2(200.0, 100.0);

	let normalized_bounds = Vec2::select(camera_track.cmpgt(Vec2::ZERO), camera_track_bounds, -camera_track_bounds);
	//If camera track is out of bounds, then bring it back towards center
	let mut camera_track = Vec2::select((camera_track+camera.vel).abs().cmpgt(camera_track_bounds), 
		normalized_bounds, 
		camera_track + camera.vel
	);
	camera_track.x *= 0.995;
	camera_track.y *= 0.995;
	if camera_track.abs().x <= 0.1 {
		camera_track.x = 0.0;
	}
	if camera_track.abs().y <= 0.1 {
		camera_track.y = 0.0;
	}
	camera_track
}

async fn load_ui_textures() -> HashMap<String, Texture2D> {
	let mut ui_textures = HashMap::<String, Texture2D>::new();
	for path in ["res/textures/fish.png", "res/textures/empty_fish.png"].into_iter() {
		if let Some((_, key)) = path.rsplit_once('/') {
			if let Ok(texture) = load_texture(path).await {
				ui_textures.insert(key.to_string(), texture);
			}
		}
	}
	return ui_textures;
}


fn global_forces(article: &mut Article) {
	if article.mass.is_finite() {
		//Gravity
		article.vel.y += 0.4;
		//Atmospheric Drag - Air Resistance
		article.vel.x -= article.vel.x * 0.005;
		article.vel.y -= article.vel.y * 0.005;
	}
}

#[macroquad::main(window_conf)]
async fn main() {
    //set_fullscreen(true);
    

	let mut articles: HashMap<String, Article> = load_articles().await;
	let mut article_keys = Vec::<String>::new();
	for (key, _) in &articles {
		article_keys.push(key.clone());
	}

	let camera_index = "Player".to_string();
	let mut camera_track = Vec2::ZERO;

	let ui_textures = load_ui_textures().await;
	


    while !is_key_down(KeyCode::Escape) {
        clear_background(WHITE);
		

		if let Some(camera) = articles.get_mut(&camera_index) {

			
			let zoom = match camera.scratchpad.get("zoom") {
				Some(z) => *z,
				None => 0.0008
			};

			camera_track = get_camera_track(camera_track, camera);
			
			set_camera(&Camera2D {
				target: vec2(camera.pos.x + camera.cog.x - (camera_track.x) + 50.0, camera.pos.y + camera.cog.y + (camera_track.y)),
				zoom: vec2(zoom, zoom * screen_width() / screen_height()),
				..Default::default()
			});
		}
		

		for index in article_keys.iter() {
			if let Some(mut article) = articles.remove(index) {
				if !article.do_destroy {
					article.tick(&mut articles);
					
					if article.mass.is_finite() {					
						global_forces(&mut article);
						article.calculate_collisions(&mut articles);
					}


					article.draw();

					articles.insert(index.clone(), article);
				}
				//If do destroy is set, article is dereferenced and freeds
			}
		}


		//Paint UI Fixtures last
		if let Some(camera) = articles.get_mut(&camera_index) {

			let player_health = match camera.scratchpad.get("health") {
				Some(h) => *h as i32,
				None => 5
			};
			let avail_player_health = match camera.scratchpad.get("avail_health") {
				Some(h) => *h as i32,
				None => 5
			};
			
			set_camera(&Camera2D {
				target: vec2(0.0, 0.0),
				zoom: vec2(0.001, 0.001 * screen_width() / screen_height()),
				..Default::default()
			});


			//Draw Player Health Bars
			for fishdex in 0..avail_player_health {
				let texture_key = if fishdex < player_health { "fish.png" } else { "empty_fish.png" };
				if let Some(fishture) = ui_textures.get(texture_key) {
					let mut x = camera.pos.x + camera.cog.x - (camera_track.x) + 50.0;
					x += (screen_width() / 2.0) - ((avail_player_health * 40) + 200) as f32;
					x += fishdex as f32 * 40.0;
					let mut y = camera.pos.y;
					y += -(screen_height() / 2.0) + 40.0;
					draw_texture(fishture, x, y, WHITE);
				}
			}
		}
		
        next_frame().await
    }
}