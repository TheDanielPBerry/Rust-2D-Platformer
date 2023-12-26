pub mod article {
	use macroquad::{math::Rect, math::Vec2, texture::{Texture2D, DrawTextureParams, draw_texture_ex}, color::{WHITE, RED}, shapes::draw_rectangle_lines};
	use std::fmt::{ Display, Formatter, Result as FmtResult };
	pub trait Element {
		fn tick(&self);
		fn draw(&self);
	}
	
	pub struct Article {
		pub name: String,	//Name should be unique to the scene
		pub texture: Option<Texture2D>,
		pub pos: Vec2,
		pub params: DrawTextureParams,
		pub bounds: Option<Vec<Rect>>,
		pub vel: Vec2,	//Velocity
		pub mass: f32,
		pub cog: Vec2,	//Center of Gravity
		pub elasticity: f32,	//Used to determine how collisions react with different materials
		pub do_destroy: bool,	//Track whether to remove an article at the end of it's next game loop
		pub tick: Option<fn(&mut Article)>,
		pub do_collide: Option<fn(axis: Vec2
			, top: &mut Article, bottom: &mut Article, intersection: &Rect) -> CollisionResult>,
		pub attached: Option<String>,	//Name of attached article, used to map items together
	}

	impl Article {
		pub fn new(src: Rect, dest: Rect, bounds: Option<Vec<Rect>>) -> Self {
			Self {
				name: String::from("Article"),
				texture: None,
				pos: Vec2::new(dest.x.to_owned(), dest.y.to_owned()),
				bounds: bounds.to_owned(),
				params: DrawTextureParams {
					dest_size: Some(Vec2::new(dest.w, dest.h)),
					source: Some(src),
					rotation: 0.0,
					flip_x: false,
					flip_y: false,
					pivot: None
				},
				do_destroy: false,
				mass: 1.0,
				vel: Vec2::new(0.0, 0.0),
				cog: Vec2::new(dest.w / 2.0, dest.h / 2.0),
				elasticity: 0.01,
				tick: None,
				do_collide: None,
				attached: None,
			}
		}

		pub async fn load_texture(&mut self, texture_path: &str) -> Option<Texture2D> {
			match macroquad::texture::load_texture(texture_path).await {
				Ok(t) => self.texture = Some(t),
				Err(e) => println!("Could not load texture: {}", e)
			};
			self.texture.clone()
		}

		pub fn draw(&self) {
			match &self.texture {
				Some(t) => draw_texture_ex(&t, self.pos.x, self.pos.y, WHITE, self.params.clone()),
				None => ()
			}
			if let Some(bounds) = &self.bounds {
				for bound in bounds.iter() {
					let bound_delta = bound.offset(self.pos);
					draw_rectangle_lines(bound_delta.x, bound_delta.y, bound_delta.w, bound_delta.h, 3.0, RED);
				}
			}
		}

		pub fn tick(&mut self) {
			match self.tick {
				Some(tick_function) => (tick_function)(self),
				None => ()
			}
 		}

		
		/**
		 * Calculate leading edge of bounds and perform appropriate collisions as needed for each article
		 */
		pub fn calculate_collisions(&mut self, articles: &mut Vec<Article>) {

			if self.vel.abs().x < 0.05 {
				self.vel.x = 0.0;
			}
			if self.vel.abs().y < 0.05 {
				self.vel.y = 0.0;
			}

			for axis in [Vec2::X, Vec2::Y].into_iter() {
				let mut did_collide: i8 = 1;
				while did_collide > 0 && did_collide < 4 {
					match &self.bounds {
						Some(bounds) => {
							let collision = bounds.iter().fold(None, |collision: Option<Collision>, top_bound: &Rect| {
								let delta = self.vel * axis;
								if delta.cmpeq(Vec2::ZERO).all() {
									return collision;
								}
								let delta_top_bound = top_bound.offset(delta).offset(self.pos);

								articles.iter_mut()
									.enumerate()
									.fold(collision, |collision: Option<Collision>, (bottom_index, bottom)| {
									
									match &bottom.bounds {
										Some(bottom_bounds) => {
											
											bottom_bounds.into_iter().fold(collision, |collision, bottom_bound| {
												match bottom_bound.offset(bottom.pos).intersect(delta_top_bound) {
													
													Some(intersection) => {
														if intersection.h == 0.0 || intersection.w == 0.0 {
															return collision;
														}
														Collision {
															intersection,
															bottom_index
														}.min_collision(axis, collision)
													}
													None => collision
												}
											})
										},
										None => collision	//If no bottom bounds maintain current collision
									}
								})
							});
							match collision {
								Some(collision) => {
									if macroquad::input::is_key_down(macroquad::miniquad::KeyCode::F) {
										
									}
									let bottom = articles.get_mut(collision.bottom_index).unwrap();
									
									let collision_result = if let Some(collide_func) = self.do_collide {
										collide_func(axis, self, bottom, &collision.intersection)
									} else {
										Self::default_collide(axis, self, bottom, &collision.intersection)
									};
									match collision_result {
										CollisionResult::Continue => {
											did_collide = 0;
											if let Some(collide_func) = bottom.do_collide {
												collide_func(axis, self, bottom, &collision.intersection);
											} else {
												Self::default_collide(axis, bottom, self, &collision.intersection);
											}
										},
										CollisionResult::DontPropagate(collide_count) => did_collide += collide_count
									};


									if axis.x == 1.0 {
										self.vel.y *= 0.8 / self.mass;
									} else if axis.y == 1.0 {
										self.vel.x *= 0.99; //Friction
									}
								},
								None => {
									if axis.y == 0.0 && self.vel.abs().y > 0.0 && did_collide == 1 {
										self.attached = None
									}
									did_collide = 0;
								}
							}
						},
						None => ()
					}
				}
			}
		}

		pub fn default_collide(axis: Vec2, a: &mut Article, b: &mut Article, intersection: &Rect) -> CollisionResult {
			
			if axis.y == 1.0 && a.pos.y < b.pos.y {
				if a.pos.y < b.pos.y {
					//If direction is down, then attach a to key of what's beneath it
					a.attached = Some(b.name.clone());
				} else {
				}
			}
			if b.mass.is_finite() {
				Self::elastic_collide(axis, a, b, intersection)
			} else {
				Self::flat_collide(axis, a, b, intersection)
			}
		}
		
		pub fn flat_collide(axis: Vec2, a: &mut Article, b: &mut Article, intersection: &Rect) -> CollisionResult {
			if axis.x == 1.0 {
				if a.vel.x < 0.0 {
					a.pos.x += a.vel.x + intersection.w;
				} else {
					a.pos.x += a.vel.x - intersection.w;
				}
				a.vel.x = 0.0;
			}
			else if axis.y == 1.0 {
				if a.vel.y < 0.0 {
					a.pos.y += a.vel.y + intersection.h;
				} else {
					a.pos.y += a.vel.y - intersection.h;
				}
				a.vel.y = 0.0;
			}
			CollisionResult::DontPropagate(-10)
		}
		
		pub fn elastic_collide(axis: Vec2, a: &mut Article, b: &mut Article, _intersection: &Rect) -> CollisionResult {
			let collision_elasticity = a.elasticity * b.elasticity;
			let mut bv = (((2.0*a.mass)/(a.mass+b.mass)*a.vel.dot(axis)) - ((a.mass-b.mass)/(a.mass+b.mass)*b.vel.dot(axis))) * collision_elasticity;
			let mut av = (((a.mass-b.mass)/(a.mass+b.mass)*(a.vel.dot(axis))) + ((2.0*b.mass)/(a.mass+b.mass)*b.vel.dot(axis))) * collision_elasticity;
			if b.mass.is_infinite() {
				av = a.vel.dot(axis) * -collision_elasticity;
			}
			if a.mass.is_infinite() {
				bv = b.vel.dot(axis) * -collision_elasticity;
			}
			if av.is_nan() {
				av = 0.0;
			}
			if bv.is_nan() {
				bv = 0.0;
			}
			a.vel = Vec2::select(axis.cmpeq(Vec2::ONE), Vec2::new(av, av), Vec2::new(a.vel.x, a.vel.y));
			b.vel = Vec2::select(axis.cmpeq(Vec2::ONE), Vec2::new(bv, bv), Vec2::new(b.vel.x, b.vel.y));
			
			CollisionResult::DontPropagate(1)
		}

	}


	
	impl Display for Article {
		fn fmt(&self, f: &mut Formatter) -> FmtResult {
			f.pad(&format!("{} {} {} {}", 
				self.pos.x, self.pos.y, 
				self.params.dest_size.unwrap().x, self.params.dest_size.unwrap().y
			))
		}
	}
	impl Display for Collision {
		fn fmt(&self, f: &mut Formatter) -> FmtResult {
			f.pad(&format!("{} {} {} {}", 
				self.intersection.x, self.intersection.y,
				self.intersection.w, self.intersection.h
			))
		}
	}

	/**
	 * Collision details for a single axis
	 */
	pub struct Collision {
		intersection: Rect,
		bottom_index: usize
	}

	impl Collision {
		pub fn min_collision(&self, axis: Vec2, a: Option<Collision>) -> Option<Collision> {
			match a {
				Some(a) => {
					if axis.x == 1.0 {
						if a.intersection.w < self.intersection.w {
							return Some(a.clone());
						} else { 
							return Some(self.clone());
						}
					}
					else if axis.y == 1.0 {
						if a.intersection.h < self.intersection.h {
							return Some(a.clone());
						} else { 
							return Some(self.clone());
						}
					}
					Some(self.clone())
				},
				None => Some(self.clone())
			}
		}
	}

	impl Clone for Collision {
		fn clone(&self) -> Self {
			Collision {
				intersection: self.intersection,
				bottom_index: self.bottom_index
			}
		}
	}

	pub enum CollisionResult {
		DontPropagate(i8),
		Continue
	}
}