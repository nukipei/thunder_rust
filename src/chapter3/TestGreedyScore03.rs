#![allow(non_snake_case)]

use rand::{Rng, SeedableRng, rngs, thread_rng};

// // 座標を保持する
#[derive(Debug, Clone, Copy)]
struct Coord {
    y: i32,
    x: i32,
}

impl Coord {
    fn new(y: i32, x: i32) -> Self {
        Coord { y, x }
    }
}

// 迷路の高さと幅
const H: usize = 3;
const W: usize = 4;
// ゲーム終了ターン
const END_TURN: usize = 4;

// 一人ゲームの例
// 1ターンに上下左右四方向のいずれかに1マスずつ進む。
// 床にあるポイントを踏むと自身のスコアとなり、床のポイントが消える。
// END_TURNの時点のスコアを高くすることが目的
#[derive(Debug, Clone)]
struct MazeState {
    character: Coord,
    points: [[i32; W]; H],
    turn: usize,
    game_score: i32,
    evaluated_score: i32,
}

impl MazeState{
    fn new(seed: Option<u64>) -> Self {
        let mut rng_for_construct: rngs::StdRng = SeedableRng::seed_from_u64(thread_rng().gen());
        if let Some(s) = seed {
            rng_for_construct = SeedableRng::seed_from_u64(s)
        }
        let character = Coord::new(rng_for_construct.gen_range(0..H as i32), rng_for_construct.gen_range(0..W as i32));

        let mut points = [[0; W]; H];   // 床のポイントを1~9で表現する

        // h*wの迷路を生成する。
        for y in 0..H {
            for x in 0..W {
                if y == character.y as usize && x == character.x as usize {
                    continue;
                }
                points[y][x] = rng_for_construct.gen_range(0..10);
             }
        }

        let turn = 0;  // 現在のターン
        let game_score = 0;  // ゲーム上で実際に得たスコア
        let evaluated_score = 0;  // 探索上で評価したスコア

        MazeState {
            character,
            points,
            turn,
            game_score,
            evaluated_score,
        }
    }

    // [どのゲームでも実装する] : ゲームの終了判定
    fn is_done(&mut self) -> bool {
        self.turn == END_TURN
    }
    // [どのゲームでも実装する] : 探索用の盤面評価をする
    fn evaluate_score(&mut self) {
        self.evaluated_score = self.game_score;
    }
    // [どのゲームでも実装する] : 指定したactionでゲームを1ターン進める
    fn advance(&mut self, action: usize) {
        let dy = [0, 0, 1, -1];
        let dx = [1, -1, 0, 0];

        self.character.x += dx[action] as i32;
        self.character.y += dy[action] as i32;

        let point = &mut self.points[self.character.y as usize][self.character.x as usize];
        if *point > 0 {
            self.game_score += *point;
            *point = 0;
        }

        self.turn += 1;
    }
    // [どのゲームでも実装する] : 現在の状況でプレイヤーが可能な行動を全て取得する
    fn legal_actions(&self) -> Vec<usize> {
        let mut actions = Vec::new();
        let dy = [0, 0, 1, -1];
        let dx = [1, -1, 0, 0];

        for action in 0..4 {
            let ty = (self.character.y + dy[action]) as usize;
            let tx = (self.character.x + dx[action]) as usize;
            if ty < H && tx < W {
                actions.push(action);
            }
        }

        actions
    }

    // [実装しなくてもよいが実装すると便利] : 現在のゲーム状況を文字列にする
    fn _to_string(&self) -> String {
        let mut result = format!("turn:\t{}\nscore:\t{}\n", self.turn, self.game_score);

        for h in 0..H {
            for w in 0..W {
                if self.character.y as usize == h && self.character.x as usize == w {
                    result.push('@');
                } else if self.points[h][w] > 0 {
                    result.push_str(&self.points[h][w].to_string());
                } else {
                    result.push('.');
                }
            }
            result.push('\n');
        }

        result
    }
}

fn greedy_action(state: &MazeState) -> usize {
    let legal_actions = state.legal_actions();
    // 絶対にありえない小さな値でベストスコアを初期化する
    let mut best_score = -1;
    // ありえない行動で初期化する
    let mut best_action = -1_isize;

    for &action in &legal_actions {
        let mut state_temp: MazeState = state.clone();
        state_temp.advance(action);
        state_temp.evaluate_score();
        if state_temp.evaluated_score > best_score {
            best_score = state_temp.evaluated_score;
            best_action = action as isize;
        }
    }
    best_action as usize
}

// ゲームをgame_number回プレイして平均スコアを表示する
fn test_ai_score(game_number: u64) {
    let mut total_score = 0;
    for _ in 0..game_number {
        let mut state: MazeState = MazeState::new(None);
        while !state.is_done() {
            state.advance(greedy_action(&state));
        }
        total_score += state.game_score;
    }
    let score_mean = total_score as f64 / game_number as f64;
    println!("Score: {}", score_mean);
}

#[allow(dead_code)]
pub fn main() {
    test_ai_score(100);
}