pub mod world {
	use crate::article::article::{Article, CollisionResult};
	use std::collections::HashMap;
	use macroquad::prelude::*;
	
	pub async fn cached_texture(article: &mut Article, texture: Result<&Texture2D, &str>) -> Option<Texture2D> {
		match texture {
			Ok(texture) => article.texture = Some(texture.to_owned()),
			Err(texture_filepath) => {
				return article.load_texture(texture_filepath).await;
			}
		}
		None
	}

	pub async fn load_articles() -> HashMap<String, Article> {
		let mut texture_map = HashMap::<&str, Texture2D>::new();
		let mut articles = HashMap::<String, Article>::new();
		
		let mut player = Article::new(
			Rect::new(0.0, 0.0, 315.0, 480.0), 
			Rect::new(1700.0,-900.0,90.0,140.0), 
			Some(vec![Rect::new(25.0, 20.0, 45.0, 88.0)])
		);
		let texture_filepath = "res/textures/penguin.png";
		if let Some(texture) = cached_texture(&mut player, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
			texture_map.insert(texture_filepath, texture);
		}
		player.name = "Player".to_string();
		player.mass = 5.0;
		player.elasticity = 0.5;
		//player.vel.x = 20.0;

		player.tick = Some(|a, articles| {
			{
				let (_, mouse_wheel_y) = mouse_wheel();
				if mouse_wheel_y != 0.0 {
					if let Some(z) = a.scratchpad.get_mut("zoom") {
						*z *= 1.1f32.powf(mouse_wheel_y/mouse_wheel_y.abs());
					} else {
						a.scratchpad.insert("zoom".to_string(), 0.001);
					}
				}
			}
			if is_key_down(KeyCode::R) {
				a.pos = Vec2::ZERO;
			}
			if is_key_down(KeyCode::S) ||  is_key_down(KeyCode::Down) {
				a.params.rotation = std::f32::consts::PI / 2.0;
				if a.friction_coefficient == 0.85 {
					if let Some(new_dest_size) = a.params.dest_size {
						a.params.dest_size = Some(Vec2::new(new_dest_size.y, new_dest_size.x));
					}
					a.vel.x*=1.5;
				}
				a.friction_coefficient = 0.99999;
			} else {
				a.params.rotation = 0.0;
				if a.friction_coefficient == 0.99999 {
					if let Some(new_dest_size) = a.params.dest_size {
						a.params.dest_size = Some(Vec2::new(new_dest_size.y, new_dest_size.x));
					}
				}
				a.friction_coefficient = 0.85;

				if is_key_down(KeyCode::A) ||  is_key_down(KeyCode::Left) {
					if let Some(_) = a.attached {
						a.vel.x -= 2.0;
					} else if a.vel.x < 0.0 {
						a.vel.x -= 0.2;
					} else if a.vel.x >= 0.0 {
						a.vel.x -= 2.0;
					}
					a.params.flip_x = false;
				}
				else if is_key_down(KeyCode::D) ||  is_key_down(KeyCode::Right) {
					if let Some(_) = a.attached {
						a.vel.x += 2.0;
					} else if a.vel.x > 0.0 {
						a.vel.x += 0.2;
					} else if a.vel.x <= 0.0 {
						a.vel.x += 2.0;
					}
					a.params.flip_x = true;
				}
			}
			if is_key_down(KeyCode::Space) ||is_key_down(KeyCode::W) ||  is_key_down(KeyCode::Up) {
				if let Some(_attachment) = &a.attached {
					a.vel.y = -23.0;
				}
				//a.remove_attachment(articles);
			}
		});
		player.do_collide = Some(|axis, a, b, intersection| {
			if axis.x == 1.0 {
				if let Some(attachment) = &a.attached {
					if attachment.as_str() == b.name.as_str() {
						return CollisionResult::DontPropagate(-10);
					}
				}
			}
			match b.do_collide {
				Some(collide_func) => collide_func(axis, b, a, intersection),
				None => Article::default_collide(axis, a, b, intersection)
			}
		});
		articles.insert(player.name.clone(), player);



		for i in 0..5 {
			//Platform 
			let mut platform = Article::new(
				Rect::new(0.0, 100.0, 400.0, 120.0),
				Rect::new(3000.0 + (i as f32*480.00),(i as f32 * 300.0),400.0,120.0), 
				Some(vec![Rect::new(0.0, 60.0, 400.0, 60.0)])
			);
			platform.name = format!("Platform-{}", i);

			let texture_filepath = "res/textures/grass.png";
			if let Some(texture) = cached_texture(&mut platform, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
				texture_map.insert(texture_filepath, texture);
			}
			platform.mass = 100_000_000.0;
			platform.elasticity = 0.0;
			platform.vel.y = 0.0;
			platform.tick = Some(|platform, articles| {
				platform.remove_attachment(articles);
				platform.vel.x = 0.0;
				if platform.vel.y >= 0.0 {
					platform.vel.y += 2.0;
					platform.attached_to = vec![];
				} else {
					platform.vel.y = -3.0;
				}
				if -100.0 > platform.pos.y {
					platform.pos.y = -100.0;
					platform.vel.y = 1.0;
				}
				if platform.pos.y > 400.0 {
					platform.pos.y = 400.0;
					platform.vel.y = -3.0;
				}
			});
			platform.do_collide = Some(|axis, a, b, intersection| {
				if axis.x == 1.0 {
					a.vel.x = 0.0;
					if let Some(attachment) = &b.attached {
						if attachment.as_str() == a.name.as_str() {
							return CollisionResult::DontPropagate(-10);
						}
					}
				}
				if axis.y == 1.0 {
					if a.pos.y > b.pos.y {
						b.vel.y -= a.vel.y;
					}
				}
				let collision_result = Article::flat_collide(axis, b, a, intersection);
				if axis.y == 1.0 {
					if a.name.contains("Platform") {
						if a.pos.y > b.pos.y {
							b.attached = Some(a.name.clone());
							if !a.attached_to.contains(&b.name) {
								a.attached_to.push(b.name.clone());
							}
						} else {
							b.attached = None;
							a.attached_to.retain(|e| e.as_str() == b.name.as_str());
							b.vel.y = 1.0;
							if a.vel.y > 0.0 {
								a.vel.y = -0.1;
							}
						}
					}
				}
				collision_result
			});
			articles.insert(platform.name.clone(), platform);
		}
		
		
		for y in -1..=0 {
			for x in (-2+y)..(2+y) {
				let mut article = Article::new(
					Rect::new(0.0, 120.0, 2056.0, 512.0), 
					Rect::new(2056.0 * x as f32,500.0 + (1000.0 * y as f32),2056.0,512.0), 
					Some(vec![Rect::new(0.0, 40.0, 2056.0, 472.0)])
				);
				article.name = format!("Grass-{}-{}", x, y);

				let texture_filepath = "res/textures/grass.png";
				if let Some(texture) = cached_texture(&mut article, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
					texture_map.insert(texture_filepath, texture);
				}
				article.mass = f32::INFINITY;
				article.elasticity = 0.0;
				articles.insert(article.name.clone(), article);
			}
		}

		for x in 8..20 {
			let mut block = Article::new(
				Rect::new(0.0, 0.0, 64.0, 64.0), 
				Rect::new(200.0 * x as f32,-100.0,64.0,64.0), 
				Some(vec![Rect::new(0.0, 0.0, 64.0, 64.0)])
			);
			block.name = format!("Block-{}", x);
			let texture_filepath = "res/textures/crate.png";
			if let Some(texture) = cached_texture(&mut block, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
				texture_map.insert(texture_filepath, texture);
			}
			block.mass = 1.0;
			block.elasticity = 1.0;
			block.friction_coefficient = 0.9;
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
			articles.insert(block.name.clone(), block);
		}
		

		for i in 1..4 {
			let x_index = i as f32 * 400.0;
			//Enemy
			let mut enemy = Article::new(
				Rect::new(0.0, 0.0, 256.0, 256.0), 
				Rect::new(x_index,0.0,256.0,256.0), 
				Some(vec![Rect::new(47.0, 33.0, 120.0, 80.0)])
			);
			let texture_filepath = "res/textures/spider.png";
			if let Some(texture) = cached_texture(&mut enemy, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
				texture_map.insert(texture_filepath, texture);
			}
			enemy.name = format!("Enemy-{}", i);
			enemy.scratchpad.insert("x-index".to_string(), x_index);
			enemy.mass = 1000000.0;
			enemy.elasticity = 1.0;
			enemy.vel.x = 20.0;
			enemy.tick = Some(|enemy, articles| {
				if enemy.vel.x >= 0.0 {
					enemy.vel.x = 6.0;
					enemy.params.flip_x = false;
				} else {
					enemy.vel.x = -6.0;
					enemy.params.flip_x = true;
				}
				if let Some(x_index) = enemy.scratchpad.get("x-index") {
					if (-500.0 + x_index) > enemy.pos.x {
						enemy.pos.x = (-500.0 + x_index);
						enemy.vel.x = 5.0;
					}
					if enemy.pos.x > (1000.0 + x_index) {
						enemy.pos.x = (1000.0 + x_index);
						enemy.vel.x = -5.0;
					}
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
		articles.insert(enemy.name.clone(), enemy);
		}

		articles
	}
}