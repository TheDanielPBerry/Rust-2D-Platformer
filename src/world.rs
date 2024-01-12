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
			Rect::new(2200.0,-200.0,90.0,140.0), 
			Some(vec![Rect::new(25.0, 20.0, 40.0, 88.0)])
		);
		let texture_filepath = "res/textures/penguin.png";
		if let Some(texture) = cached_texture(&mut player, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
			texture_map.insert(texture_filepath, texture);
		}
		player.name = "Player".to_string();
		player.mass = 5.0;
		player.elasticity = 0.5;
		player.cog = vec2(44.5, 66.0);

		player.tick = Some(|a, _articles| {
			{
				let (_, mouse_wheel_y) = mouse_wheel();
				if mouse_wheel_y != 0.0 {
					if let Some(z) = a.scratchpad.get_mut("zoom") {
						*z *= 1.1f32.powf(mouse_wheel_y/mouse_wheel_y.abs());
					} else {
						a.scratchpad.insert("zoom".to_string(), 0.0008);
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
						a.params.dest_size = Some(vec2(new_dest_size.y, new_dest_size.x));
					}
					a.vel.x*=1.5;
				}
				a.friction_coefficient = 0.99999;
			} else {
				a.params.rotation = 0.0;
				if a.friction_coefficient == 0.99999 {
					if let Some(new_dest_size) = a.params.dest_size {
						a.params.dest_size = Some(vec2(new_dest_size.y, new_dest_size.x));
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
					a.set_direction(-Vec2::X);
				}
				else if is_key_down(KeyCode::D) ||  is_key_down(KeyCode::Right) {
					if let Some(_) = a.attached {
						a.vel.x += 2.0;
					} else if a.vel.x > 0.0 {
						a.vel.x += 0.2;
					} else if a.vel.x <= 0.0 {
						a.vel.x += 2.0;
					}
					a.set_direction(Vec2::X);
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
				Rect::new(0.0, 0.0, 256.0, 128.0), 
				Rect::new(x_index,0.0,256.0,128.0), 
				Some(vec![Rect::new(47.0, 33.0, 129.0, 62.0)])
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
					enemy.set_direction(Vec2::X);
				} else {
					enemy.vel.x = -6.0;
					enemy.set_direction(-Vec2::X);
				}
				if let Some(x_index) = enemy.scratchpad.get("x-index") {
					if (-500.0 + x_index) > enemy.pos.x {
						enemy.pos.x = -500.0 + x_index;
						enemy.vel.x = 5.0;
					}
					if enemy.pos.x > (1000.0 + x_index) {
						enemy.pos.x = 1000.0 + x_index;
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

		
		//Fisherman
		let mut fisherman = Article::new(
			Rect::new(0.0, 0.0, 384.0, 512.0), 
			Rect::new(1800.0, -1000.0, 384.0, 512.0), 
			Some(vec![
				Rect::new(114.0, 358.0, 73.0, 98.0),	//Torso
				Rect::new(132.0, 457.0, 41.0, 48.0),	//Legs
				Rect::new(138.0, 309.0, 35.0, 49.0)		//Head
			])
		);
		let texture_filepath = "res/textures/fisherman_spritesheet.png";
		if let Some(texture) = cached_texture(&mut fisherman, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
			texture_map.insert(texture_filepath, texture);
		}
		
		fisherman.name = format!("fisherman-{}", 0);
		fisherman.scratchpad.insert("x-index".to_string(), -100.0);
		fisherman.scratchpad.insert("player-x".to_string(), -100.0);
		fisherman.scratchpad.insert("player-x".to_string(), -100.0);
		fisherman.scratchpad.insert("status".to_string(), 0.0);
		fisherman.mass = 10_000_000.0;
		fisherman.elasticity = 1.0;
		fisherman.set_direction(Vec2::X);
		

	
		let mut lure  = Article::new(
			Rect::new(0.0, 0.0, 32.0, 32.0), 
			Rect::new(100_00.0, 100_000.0,32.0,32.0), 
			Some(vec![Rect::new(0.0, 0.0, 25.0, 25.0)])
		);
		lure.name = format!("lure-{}", fisherman.name.clone());
		let texture_filepath = "res/textures/lure.png";
		if let Some(texture) = cached_texture(&mut lure, texture_map.get(texture_filepath).ok_or(texture_filepath)).await {
			texture_map.insert(texture_filepath, texture);
		}
		lure.elasticity = 0.5;
		lure.mass = 1.0;
		lure.do_collide = Some(|axis, a, b, intersection| {
			let mut hidden = 1.0;
			if a.name.contains("lure") {
				if let Some(is_hidden) = a.scratchpad.get("hidden") {
					hidden = *is_hidden;
				}
				if hidden == 0.0 {
					return Article::elastic_collide(axis, a, b, intersection);
				}
			}
			return CollisionResult::DontPropagate(10);
		});
		lure.draw = Some(|lure| -> bool {
			let mut hidden = 1.0;
			if let Some(is_hidden) = lure.scratchpad.get("hidden") {
				hidden = *is_hidden;
			}
			if hidden == 0.0 {
				let mut pole = lure.pos.clone();
				if let Some(fisherman_x) = lure.scratchpad.get("fisherman-pole-x") {
					pole.x = *fisherman_x;
				}
				if let Some(fisherman_y) = lure.scratchpad.get("fisherman-pole-y") {
					pole.y = *fisherman_y;
				}
				draw_line(pole.x, pole.y, lure.pos.x+5.0, lure.pos.y+5.0, 1.0, BLACK);
			}
			return hidden == 0.0;
		});
		lure.tick = Some(|lure, articles| {
			if let Some(player) = articles.get_mut("Player") {
				if (player.pos.x - lure.pos.x).abs() < 300.0 {
					//Need to calculate lure distance and remaining velocity
					lure.vel.x *= 0.89 + (player.pos.x - lure.pos.x).abs() / 3000.0;
				}
			}
		});
		articles.insert(lure.name.clone(), lure);
		

		fisherman.tick = Some(|fisherman, articles| {
			if let Some(s) = fisherman.scratchpad.remove("status") {
				let mut status = s;
				if let Some(player) = articles.get_mut("Player") {
					if player.pos.distance(fisherman.pos) < 1800.0 {
						status += 1.0;
						if status == 90.0 {
							//Start Casting
							fisherman.scratchpad.insert("player-x".to_string(), player.pos.x);
							fisherman.scratchpad.insert("player-y".to_string(), player.pos.y);
						}
					} else {
						status = 0.0;
					}
				}

				let lure_name = format!("lure-{}", fisherman.name.clone());
				if let Some(lure) = articles.get_mut(lure_name.as_str()) {
					if status == 90.0  {
						//Position lure above fisherman
						lure.pos.x = fisherman.pos.x + 281.0;
						lure.pos.y = fisherman.pos.y + 197.0;
						lure.vel.y = -15.0;
						//Calculate x vel to run into player
						if let Some(player_x) = fisherman.scratchpad.get("player-x") {
							lure.vel.x = (*player_x - lure.pos.x)/50.0;
						}
						lure.scratchpad.insert("fisherman-pole-x".to_string(), fisherman.pos.x + 281.0);
						lure.scratchpad.insert("fisherman-pole-y".to_string(), fisherman.pos.y + 197.0);
						lure.scratchpad.insert("hidden".to_string(), 0.0);

					} else if [20.0, 50.0, 60.0, 70.0, 80.0, 90.0].contains(&status) {
						fisherman.increment_frame(Vec2::X);
					} else if status > 400.0 {
						status = 1.0;
						fisherman.set_frame(vec2(0.0, 0.0));
						lure.scratchpad.insert("hidden".to_string(), 1.0);
					} else if status == 0.0 {
						//Reset lure
						lure.scratchpad.insert("hidden".to_string(), 1.0);
						fisherman.set_frame(Vec2::ZERO);


						/* 
						if fisherman.vel.x >= 0.0 {
							fisherman.vel.x = 3.0;
							fisherman.params.flip_x = true;
						} else {
							fisherman.vel.x = -3.0;
							fisherman.params.flip_x = false;
						}
						if let Some(x_index) = fisherman.scratchpad.get("x-index") {
							if (-500.0 + x_index) > fisherman.pos.x {
								fisherman.pos.x = -500.0 + x_index;
								fisherman.vel.x = 3.0;
							}
							if fisherman.pos.x > (1000.0 + x_index) {
								fisherman.pos.x = 1000.0 + x_index;
								fisherman.vel.x = -3.0;
							}
						}
						*/
					}
				}
				fisherman.scratchpad.insert("status".to_string(), status);
			}
		});
		fisherman.do_collide = Some(|axis, a, b, intersection| {
			if axis.y == 1.0 && b.vel.y > 0.1 {
				a.vel.y -= 15.0;
			}
			
			let collision_result = Article::default_collide(axis, a, b, intersection);
			if axis.y == 1.0 && b.vel.y > 0.1 {
				a.vel.y += 15.0;
			}
			if axis.x == 1.0 {
				a.vel.x *= -1.0;
			}
			collision_result
		});

		articles.insert(fisherman.name.clone(), fisherman);


		articles
	}

	

}