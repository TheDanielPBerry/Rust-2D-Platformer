use std::{cmp::Ordering, collections::HashMap};

use macroquad::prelude::*;
use macroquad::math::Rect;


mod article;
use article::article::{Article, CollisionResult};

const ZOOM: f32 = 0.001;

fn window_conf() -> Conf {
	Conf {
		window_title: "Game".to_owned(),
		fullscreen: false,
		..Default::default()
	}
}

async fn load_texture(article: &mut Article, texture: Result<&Texture2D, &str>) -> Option<Texture2D> {
	match texture {
		Ok(texture) => article.texture = Some(texture.to_owned()),
		Err(texture_filepath) => {
			return article.load_texture(texture_filepath).await;
		}
	}
	None
}

async fn load_articles() -> Vec::<Article> {
	let mut texture_map = HashMap::<&str, Texture2D>::new();
	let mut articles = Vec::<Article>::new();

	let mut player = Article::new(
		Rect::new(0.0, 0.0, 315.0, 480.0), 
		Rect::new(100.0,100.0,90.0,140.0), 
		Some(vec![Rect::new(25.0, 20.0, 45.0, 88.0)])
	);
	let texture_filepath = "res/textures/penguin.png";
	if let Some(texture) = load_texture(&mut player, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
		texture_map.insert(texture_filepath, texture);
	}
	player.name = "Player".to_string();
	player.elasticity = 0.5;
	player.vel.x = 20.0;

	player.tick = Some(|a| {
		if is_key_down(KeyCode::R) {
			a.pos = Vec2::ZERO;
		}
		if is_key_down(KeyCode::S) ||  is_key_down(KeyCode::Down) {
			a.vel.x *= 0.8;
		}
		if is_key_down(KeyCode::A) ||  is_key_down(KeyCode::Left) {
			if a.vel.abs().y <= 0.01 {
				a.vel.x -= 2.0;
			} else if a.vel.x < 0.0 {
				a.vel.x -= 0.2;
			} else if a.vel.x >= 0.0 {
				a.vel.x -= 2.0;
			}
			a.params.flip_x = false;
		}
		else if is_key_down(KeyCode::D) ||  is_key_down(KeyCode::Right) {
			if a.vel.abs().y <= 0.01 {
				a.vel.x += 2.0;
			} else if a.vel.x > 0.0 {
				a.vel.x += 0.2;
			} else if a.vel.x <= 0.0 {
				a.vel.x += 2.0;
			}
			a.params.flip_x = true;
		}
		if is_key_released(KeyCode::Space) ||is_key_down(KeyCode::W) ||  is_key_released(KeyCode::Up) {
			if let Some(_attachment) = &a.attached {
				a.vel.y = -25.0;
			}
		}
	});
	player.do_collide = Some(|axis, a, b, intersection| {
		if axis.x == 1.0 {
			if let Some(attachment) = &a.attached {
				if let Ordering::Equal = attachment.cmp(&b.name) {
					return CollisionResult::DontPropagate(-10);
				}
			}
			if is_key_down(KeyCode::F) {
				println!("{}", intersection.w);
			}
		}
		match b.do_collide {
			Some(collide_func) => collide_func(axis, b, a, intersection),
			None => Article::default_collide(axis, a, b, intersection)
		}
	});
	articles.push(player);



	
	//Platform 
	let mut platform = Article::new(
		Rect::new(0.0, 100.0, 400.0, 120.0),
		Rect::new(1800.0,0.0,400.0,120.0), 
		Some(vec![Rect::new(0.0, 60.0, 400.0, 60.0)])
	);
	platform.name = String::from("Platform-1");

	let texture_filepath = "res/textures/grass.png";
	if let Some(texture) = load_texture(&mut platform, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
		texture_map.insert(texture_filepath, texture);
	}
	platform.mass = f32::INFINITY;
	platform.elasticity = 0.0;
	platform.vel.y = 0.0;
	platform.tick = Some(|a: &mut Article| {
		a.vel.x = 0.0;
		if a.vel.y >= 0.0 {
			a.vel.y += 1.0;
		} else {
			a.vel.y = -3.0;
		}
		if -100.0 > a.pos.y {
			a.pos.y = -100.0;
			a.vel.y = 1.0;
		}
		if a.pos.y > 400.0 {
			a.pos.y = 400.0;
			a.vel.y = -3.0;
		}
	});
	platform.do_collide = Some(|axis, a, b, intersection| {
		if axis.x == 1.0 {
			a.vel.x = 0.0;
		}
		if let Some(attachment) = &b.attached {
			if attachment.as_str() == a.name.as_str() {
				//return CollisionResult::DontPropagate(-10);
			}
		}
		if axis.y == 1.0 {
			if a.pos.y > b.pos.y {
				b.vel.y += -a.vel.y;
			}
		}
		let collision_result = Article::flat_collide(axis, b, a, intersection);
		if axis.y == 1.0 {
			if a.pos.y > b.pos.y {
				b.attached = Some(a.name.clone());
				b.vel.y = a.vel.y;
			} else {
				b.attached = None;
				b.vel.y = 0.0;
				a.vel.y *= -1.0;
			}
		}
		collision_result
	});
	articles.push(platform);
	
	
	for x in -2..4 {
		for y in 1..2 {
			let mut article = Article::new(
				Rect::new(0.0, 120.0, 2056.0, 512.0), 
				Rect::new(2056.0 * x as f32,600.0 * y as f32,2056.0,512.0), 
				Some(vec![Rect::new(0.0, 40.0, 2056.0, 472.0)])
			);
			article.name = format!("Grass-{}-{}", x, y);

			let texture_filepath = "res/textures/grass.png";
			if let Some(texture) = load_texture(&mut article, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
				texture_map.insert(texture_filepath, texture);
			}
			article.mass = f32::INFINITY;
			article.elasticity = 0.0;
			articles.push(article);
		}
	}

	for x in 1..3 {
		let mut block = Article::new(
			Rect::new(0.0, 100.0, 200.0, 320.0), 
			Rect::new(2100.0 * x as f32,0.0,200.0,160.0), 
			Some(vec![Rect::new(0.0, 20.0, 200.0, 140.0)])
		);
		block.name = format!("Block-{}", x);
		let texture_filepath = "res/textures/grass.png";
		if let Some(texture) = load_texture(&mut block, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
			texture_map.insert(texture_filepath, texture);
		}
		block.mass = x as f32;
		block.elasticity = 1.0;
		block.do_collide = Some(|axis, a, b, intersection| {
			if axis.x == 1.0 {
				if let Some(attachment) = &a.attached {
					if attachment.as_str() == b.name.as_str() {
						return CollisionResult::DontPropagate(-10);
					}
				}
				return Article::elastic_collide(axis, a, b, intersection);
			}
			if a.pos.y < b.pos.y {
				a.attached = Some(b.name.clone());
				Article::flat_collide(axis, a, b, intersection)
			} else {
				b.attached = Some(a.name.clone());
				Article::flat_collide(axis, b, a, intersection)
			}
		});
		articles.push(block);
	}
	

	//Enemy
	let mut enemy = Article::new(
		Rect::new(0.0, 0.0, 315.0, 480.0), 
		Rect::new(200.0,100.0,90.0,140.0), 
		Some(vec![Rect::new(25.0, 20.0, 45.0, 88.0)])
	);
	let texture_filepath = "res/textures/penguin.png";
	if let Some(texture) = load_texture(&mut enemy, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
		texture_map.insert(texture_filepath, texture);
	}
	enemy.name = String::from("Enemy-1");
	enemy.mass = 10000.0;
	enemy.elasticity = 1.0;
	enemy.vel.x = 20.0;
	enemy.tick = Some(|a: &mut Article| {
		if a.vel.x >= 0.0 {
			a.vel.x = 3.0;
		} else {
			a.vel.x = -3.0;
		}
		if -200.0 > a.pos.x {
			a.pos.x = -200.0;
			a.vel.x = 3.0;
		}
		if a.pos.x > 1000.0 {
			a.pos.x = 1000.0;
			a.vel.x = -3.0;
		}
	});
	enemy.do_collide = Some(|axis, a, b, intersection| {
		if axis.y == 1.0 && b.vel.y > 0.1 {
			a.do_destroy = true;
		}
		let collision_result = Article::default_collide(axis, a, b, intersection);
		if axis.x == 1.0 {
			a.vel.x *= -1.0;
		}
		collision_result
	});
	articles.push(enemy);

	articles
}

fn global_forces(article: &mut Article) {
	if article.mass.is_finite() {
		//Gravity
		(*article).vel.y += 0.4;
		//Atmospheric Drag - Air Resistance
		(*article).vel.x -= article.vel.x * 0.01;
		(*article).vel.y -= article.vel.y * 0.01;
	}
}

#[macroquad::main(window_conf)]
async fn main() {
    //set_fullscreen(true);


	let mut articles = load_articles().await;
	let camera_index: usize = 0;
	
	let mut camera_track = Vec2::ZERO;
	let camera_track_bounds = Vec2::new(200.0, 100.0);
    while !is_key_down(KeyCode::Escape) {
        clear_background(WHITE);
		
		{
			let camera = articles.get_mut(camera_index).unwrap();
			
			let normalized_bounds = Vec2::select(camera_track.cmpgt(Vec2::ZERO), camera_track_bounds, -camera_track_bounds);
			//If camera track is out of bounds, then bring it back towards center
			camera_track = Vec2::select((camera_track+camera.vel).abs().cmpgt(camera_track_bounds), 
				normalized_bounds, 
				camera_track + camera.vel
			);
			camera_track.y *= 0.99;
			
			set_camera(&Camera2D {
				target: vec2(camera.pos.x + camera.cog.x - (camera_track.x) + 100.0, camera.pos.y + camera.cog.y + (camera_track.y)),
				zoom: vec2(ZOOM, ZOOM * screen_width() / screen_height()),
				..Default::default()
			});
		}

		for index in 0..articles.len() {
			let mut article = articles.remove(index);
			if !article.do_destroy {
				article.tick();
				global_forces(&mut article);

				article.calculate_collisions(&mut articles);

				article.pos.x += article.vel.x;
				article.pos.y += article.vel.y;

				article.draw();

				articles.insert(index, article);
			}
			//Article is dereferenced and memory is freed
		}
		
        next_frame().await
    }
}