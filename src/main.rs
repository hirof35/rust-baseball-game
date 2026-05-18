use macroquad::prelude::*;
use ::rand::Rng;

#[derive(PartialEq)]
enum SceneType { Title, Playing, Result }

#[derive(PartialEq)]
enum GameState { Ready, Pitching, Batted }

struct Ball {
    pos: Vec2,
    velocity: Vec2,
    spin: f32,
    drop: f32,
}

struct Fielder {
    home_pos: Vec2,
    pos: Vec2,
    speed: f32,
}

impl Fielder {
    fn update(&mut self, target: Vec2, active: bool, delta: f32) {
        let dest = if active { target } else { self.home_pos };
        if self.pos.distance(dest) > 5.0 {
            self.pos += (dest - self.pos).normalize() * self.speed * delta;
        }
    }
}

struct ScoreBoard {
    inning: i32,
    is_top: bool,
    score: [i32; 2],
    outs: i32,
    strikes: i32,
    balls: i32,
    max_inning: i32,
}

impl ScoreBoard {
    fn new() -> Self {
        Self { inning: 1, is_top: true, score: [0, 0], outs: 0, strikes: 0, balls: 0, max_inning: 3 }
    }
    fn reset(&mut self) { *self = Self::new(); }
    fn add_score(&mut self, pts: i32) -> bool {
        let idx = if self.is_top { 0 } else { 1 };
        self.score[idx] += pts;
        !self.is_top && self.inning >= self.max_inning && self.score[1] > self.score[0]
    }
    fn next_inning(&mut self) {
        self.outs = 0; self.strikes = 0; self.balls = 0;
        if !self.is_top { self.inning += 1; }
        self.is_top = !self.is_top;
    }
}

#[macroquad::main("Rust Baseball Game")]
async fn main() {
    let mut scene = SceneType::Title;
    let mut state = GameState::Ready;
    let mut sb = ScoreBoard::new();
    let mut ball = Ball { pos: Vec2::ZERO, velocity: Vec2::ZERO, spin: 0.0, drop: 1.0 };
    let mut rng = ::rand::thread_rng();

    let mut fielders = vec![
        Fielder { home_pos: vec2(400.0, 150.0), pos: vec2(400.0, 150.0), speed: 90.0 },
        Fielder { home_pos: vec2(200.0, 250.0), pos: vec2(200.0, 250.0), speed: 70.0 },
        Fielder { home_pos: vec2(600.0, 250.0), pos: vec2(600.0, 250.0), speed: 70.0 },
        Fielder { home_pos: vec2(350.0, 380.0), pos: vec2(350.0, 380.0), speed: 110.0 },
        Fielder { home_pos: vec2(450.0, 380.0), pos: vec2(450.0, 380.0), speed: 110.0 },
    ];

    let mut p_anim: f32 = 0.0;
    let mut b_anim: f32 = 1.0;
    let mut batter_x: f32 = 480.0;
    let mut result_text = String::new();

    loop {
        let delta = get_frame_time();
        clear_background(Color::from_rgba(30, 100, 30, 255));

        match scene {
            SceneType::Title => {
                draw_text("Rust Baseball", 200.0, 200.0, 60.0, YELLOW);
                draw_text("Press ENTER to Play", 280.0, 350.0, 25.0, WHITE);
                if is_key_pressed(KeyCode::Enter) {
                    sb.reset(); scene = SceneType::Playing; state = GameState::Ready;
                }
            }
            SceneType::Playing => {
                // グラウンド（土）とホームベースの描画
                draw_rectangle(0.0, 450.0, 800.0, 150.0, Color::from_rgba(100, 80, 50, 255));
                draw_line(300.0, 500.0, 500.0, 500.0, 5.0, WHITE);

                // --- バッターの左右移動入力 ---
                if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                    batter_x -= 250.0_f32 * delta; // 移動を少しスムーズに
                }
                if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                    batter_x += 250.0_f32 * delta;
                }
                batter_x = batter_x.clamp(250.0_f32, 650.0_f32); // 移動範囲を少し広げました

                // --- 投球・打撃ロジック ---
                if state == GameState::Ready {
                    if is_key_pressed(KeyCode::Space) {
                        state = GameState::Pitching; p_anim = 0.0; b_anim = 1.0; ball.velocity = Vec2::ZERO;
                    }
                }
                else if state == GameState::Pitching {
                    p_anim += delta * 1.5_f32;
                    if p_anim >= 0.5_f32 && ball.velocity == Vec2::ZERO {
                        if rng.gen_bool(0.6) {
                            ball = Ball { pos: vec2(400.0, 180.0), velocity: vec2(rng.gen_range(-25.0..25.0), 450.0), spin: 0.0, drop: 1.0 };
                        } else {
                            ball = Ball { pos: vec2(400.0, 180.0), velocity: vec2(rng.gen_range(30.0..70.0), 380.0), spin: rng.gen_range(-7.0..-4.0), drop: 1.3 };
                        }
                    }
                    if ball.velocity != Vec2::ZERO {
                        ball.velocity.x += ball.spin * 60.0_f32 * delta;
                        ball.pos += ball.velocity * delta;

                        // 左クリックでスイング開始
                        if is_mouse_button_pressed(MouseButton::Left) && b_anim >= 1.0_f32 { b_anim = 0.0; }

                        // ★ 判定用のバット位置計算を、描画側と完全に同期（Y座標を 480.0 に統一）
                        let bat_start_calc = vec2(batter_x - 10.0, 480.0);
                        let t_calc: f32 = b_anim.min(1.0_f32);
                        let bat_angle_calc: f32 = 140.0_f32 + (130.0_f32 * t_calc);
                        let rad_calc: f32 = bat_angle_calc.to_radians();
                        let bat_end_calc = bat_start_calc + vec2(70.0_f32 * rad_calc.cos(), 70.0_f32 * rad_calc.sin());

                        // ★ スイング中（b_animが0.0〜0.7の間）に判定を行う。判定距離を 25.0 -> 40.0 に拡大！
                        // ★ 打撃判定のブロックを以下に差し替え
                        if b_anim < 0.7_f32 && ball.pos.distance(bat_end_calc) < 40.0_f32 {
                            // 1. スイングのタイミング（b_anim）から打球の基本角度を計算
                            // b_animが小さい（スイング初期）＝レフト方向 / 大きい＝ライト方向
                            let hit_angle_deg = 200.0_f32 + (b_anim * 120.0_f32); 
                            let hit_rad = hit_angle_deg.to_radians();
                            
                            // 2. 打球の速度ベクトルを設定（上方向へ飛ばす）
                            ball.velocity = vec2(hit_rad.cos(), hit_rad.sin()) * 650.0;
                            
                            // 3. スピンは「ボールとバットの上下のズレ」でマイルドに計算
                            // バットのやや上で捉えたらライナー・フライ（適度なバックスピン）
                            let off_center_y = bat_end_calc.y - ball.pos.y;
                            ball.spin = off_center_y.clamp(-10.0, 10.0) * 0.8; 
                            
                            state = GameState::Batted;
                        
                        } else if ball.pos.y > 560.0 {
                            // 見逃し・空振り
                            sb.strikes += 1;
                            if sb.strikes >= 3 { sb.outs += 1; sb.strikes = 0; if sb.outs >= 3 { sb.next_inning(); } }
                            state = GameState::Ready;
                        }
                    }
                }
                else if state == GameState::Batted {
                    ball.velocity.y += 300.0_f32 * delta;
                    let lift_dir = vec2(-ball.velocity.y, ball.velocity.x).normalize();
                    ball.velocity += lift_dir * (ball.spin * 0.5_f32);
                    ball.pos += ball.velocity * delta;

                    for f in &mut fielders {
                        if f.pos.distance(ball.pos) < 20.0 {
                            sb.outs += 1; if sb.outs >= 3 { sb.next_inning(); }
                            state = GameState::Ready;
                        }
                    }

                    if ball.pos.y < 0.0 || ball.pos.y > 600.0 || ball.pos.x < 0.0 || ball.pos.x > 800.0 {
                        if sb.add_score(1) { scene = SceneType::Result; result_text = "サヨナラ勝ち！".to_string(); }
                        state = GameState::Ready;
                    }
                }

                if b_anim < 1.0_f32 { b_anim += delta * 5.5_f32; } // スイング速度を微増

                // --- 描画 ---
                for f in &mut fielders {
                    f.update(ball.pos, state == GameState::Batted, delta);
                    draw_circle(f.pos.x, f.pos.y, 8.0, BLUE);
                }

                if state != GameState::Ready && p_anim >= 0.5_f32 {
                    draw_circle(ball.pos.x, ball.pos.y, 8.0, WHITE);
                }
                
                let batter_pos = vec2(batter_x, 500.0);
                draw_circle(batter_pos.x, batter_pos.y - 40.0, 12.0, BEIGE);
                draw_line(batter_pos.x, batter_pos.y - 28.0, batter_pos.x, batter_pos.y, 6.0, BLUE);

                let bat_start = vec2(batter_pos.x - 10.0, batter_pos.y - 20.0); // 500 - 20 = 480 で上記と一致
                let t: f32 = b_anim.min(1.0_f32);
                let bat_angle: f32 = 140.0_f32 + (130.0_f32 * t); 
                let rad: f32 = bat_angle.to_radians();
                let bat_end = bat_start + vec2(70.0_f32 * rad.cos(), 70.0_f32 * rad.sin());

                draw_line(bat_start.x, bat_start.y, bat_end.x, bat_end.y, 5.0, ORANGE);

                // UI表示
                let ui_text = format!("{}Inning {}  {} - {}  S:{} B:{} O:{}", sb.inning, if sb.is_top { "Top" } else { "Bot" }, sb.score[0], sb.score[1], sb.strikes, sb.balls, sb.outs);
                draw_text(&ui_text, 20.0, 40.0, 25.0, WHITE);
                draw_text("A / D (Left / Right): Move Batter", 20.0, 70.0, 18.0, LIGHTGRAY);
            }
            // ★ ここを復元しました
            SceneType::Result => {
                draw_text("GAME SET", 280.0, 200.0, 60.0, WHITE);
                draw_text(&result_text, 340.0, 300.0, 30.0, YELLOW);
                draw_text("Press ESC to Title", 300.0, 450.0, 25.0, WHITE);
                if is_key_pressed(KeyCode::Escape) { scene = SceneType::Title; }
            }
        }

        next_frame().await
    }
}