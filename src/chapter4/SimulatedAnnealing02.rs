#![allow(non_snake_case)]

use once_cell::sync::Lazy;
use std::sync::Mutex;

use rand::{Rng, SeedableRng, rngs, thread_rng};

const H: usize = 5;        // 迷路の高さ
const W: usize = 5;        // 迷路の幅
const END_TURN: usize = 5;  // ゲーム終了ターン
const CHARACTER_N: usize = 3; // キャラクターの数

type ScoreType = i64;
const INF: ScoreType = 1000000000;


// グローバルな乱数生成器
static RNG:  Lazy<Mutex<rngs::StdRng>> = Lazy::new(|| {
    // ここにシード値を指定して初期化する
    let seed = 42; // 任意のシード値を指定
    let rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
    Mutex::new(rng)
});
static RNG_FOR_ANMEAL:  Lazy<Mutex<rngs::StdRng>> = Lazy::new(|| {
    // ここにシード値を指定して初期化する
    let seed = 41; // 任意のシード値を指定
    let rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
    Mutex::new(rng)
});


// 座標を保持する
#[derive(Clone, Copy)]
struct Coord {
    y: usize,
    x: usize,
}

// 自動一人ゲームの例
// キャラクターは1マス先の最もポイントが高い床に自動で移動する。
// 合法手の中でスコアが同値のものがある場合、右、左、下、上の順で行動が優先される。
// 1ターンに上下左右四方向のいずれかに壁のない場所に1マスずつ進む。
// 床にあるポイントを踏むと自身のスコアとなり、床のポイントが消える。
// END_TURNの時点のスコアを高くすることを目的とし、
// ゲームに介入できる要素として、初期状態でのキャラクターをどこに配置するかを選択できる。
// どのようにキャラクターを配置すると最終スコアが高くなるかを考えるゲーム。
# [derive (Clone)]
struct AutoMoveMazeState {
    points: [[usize; W]; H], // 床のポイントを1~9で表現する
    turn: usize,             // 現在のターン
    characters: [Coord; CHARACTER_N], // CHARACTER_N体のキャラクター
    game_score: usize,       // ゲーム上で実際に得たスコア
    // evaluated_score: ScoreType, // 探索上で評価したスコア
}

impl AutoMoveMazeState {
    // h*wの迷路を生成する。
    fn new(seed: Option<usize>) -> Self {

        let mut rng: rngs::StdRng = SeedableRng::seed_from_u64(thread_rng().gen());
        if let Some(s) = seed  {
            rng = SeedableRng::seed_from_u64(s as u64)
        }
        let mut points = [[0; W]; H];
        for y in 0..H {
            for x in 0..W {
                points[y][x] = rng.gen_range(1..=9);
            }
        }

        AutoMoveMazeState {
            points,
            turn: 0,
            characters: [Coord { y: 0, x: 0 }; CHARACTER_N],
            game_score: 0,
            // evaluated_score: 0,
        }
    }

    // 指定位置に指定キャラクターを配置する。
    fn set_character(&mut self, character_id: usize, y: usize, x: usize) {
        self.characters[character_id].y = y;
        self.characters[character_id].x = x;
    }

    // ゲームの終了判定
    fn is_done(&self) -> bool {
        self.turn == END_TURN
    }

    // ゲームを1ターン進める。
    fn advance(&mut self) {
        for character_id in 0..CHARACTER_N {
            self.move_player(character_id);
        }
        for character in &self.characters {
            let point = &mut self.points[character.y][character.x];
            self.game_score += *point;
            *point = 0;
        }
        self.turn += 1;
    }

    // 指定キャラクターを移動させる。
    fn move_player(&mut self, character_id: usize) {
        let character = &mut self.characters[character_id];
        let mut best_point: ScoreType = -INF;

        // 盤面の範囲内となるような移動先を取得する。
        let mut legal_action = Vec::with_capacity(4);
        for action in 0..4 {
            let ty = character.y as isize + dy[action];
            let tx = character.x as isize + dx[action];

            if ty >= 0 && ty < H as isize && tx >= 0 && tx < W as isize {
                legal_action.push(action);
            }
        }

        let mut best_action_index = legal_action[0];

        for action in legal_action {
            let ty = character.y as isize + dy[action];
            let tx = character.x as isize + dx[action];

            let point = self.points[ty as usize][tx as usize] as ScoreType;

            if point > best_point {
                best_point = point;
                best_action_index = action;
            }
        }

        character.y = (character.y as isize + dy[best_action_index]) as usize;
        character.x = (character.x as isize + dx[best_action_index]) as usize;
    }

    // 現在のゲーム状況を文字列にする
    fn to_string(&self) -> String {
        let mut s = format!("turn:\t{}\n", self.turn);
        s += &format!("score:\t{}\n", self.game_score);

        for h in 0..H {
            for w in 0..W {
                let mut is_written = false;

                for character in &self.characters {
                    if character.y == h && character.x == w {
                        s += "@";
                        is_written = true;
                        break;
                    }
                }

                if !is_written {
                    if self.points[h][w] > 0 {
                        s += &self.points[h][w].to_string();
                    } else {
                        s += ".";
                    }
                }
            }
            s += "\n";
        }

        s
    }

    // スコア計算をする。(toStringを実装しない場合は引数is_printとそれの不随する処理は不要)
    fn get_score(&mut self, is_print: bool) -> ScoreType {
        let mut tmp_state = self.clone();

        for character in &self.characters {
            let point = &mut tmp_state.points[character.y][character.x];
            *point = 0;
        }

        while !tmp_state.is_done() {
            tmp_state.advance();
            if is_print {
                println!("{}", tmp_state.to_string());
            }
        }

        tmp_state.game_score as ScoreType
    }

    // 初期化する
    fn init(&mut self){
        for character_id in 0..CHARACTER_N {
            let y = RNG.lock().unwrap().gen_range(0..H);
            let x = RNG.lock().unwrap().gen_range(0..W);

            self.set_character(character_id, y, x);
        }
    }

    // 状態遷移する
    fn transition(&mut self){
        let character_id = RNG.lock().unwrap().gen_range(0..CHARACTER_N);
        let character = &mut self.characters[character_id];
        character.y = RNG.lock().unwrap().gen_range(0..H);
        character.x = RNG.lock().unwrap().gen_range(0..W);
    }
}

#[allow(non_upper_case_globals)]
const dy: [isize; 4] = [0, 0, 1, -1];

#[allow(non_upper_case_globals)]
const dx: [isize; 4] = [1, -1, 0, 0];

type AIFunction = fn(&AutoMoveMazeState) -> AutoMoveMazeState;

fn hill_climb(state: &AutoMoveMazeState, number: isize) -> AutoMoveMazeState {
    let mut now_state = state.clone();
    now_state.init();
    let mut best_score: ScoreType = now_state.get_score(false);
    for _ in 0..number {
        let mut next_state = now_state.clone();
        next_state.transition();
        let next_score: ScoreType = next_state.get_score(false);
        if next_score > best_score {
            best_score = next_score;
            now_state = next_state;
        }
    }

    now_state
}


fn simulated_annealing(state: &AutoMoveMazeState, number: usize, start_temp: f64, end_temp: f64) -> AutoMoveMazeState {
    let mut now_state = state.clone();
    now_state.init();
    let mut best_score = now_state.get_score(false) as ScoreType;
    let mut now_score = best_score as ScoreType;
    let mut best_state = now_state.clone();

    let mut rng = RNG_FOR_ANMEAL.lock().unwrap();

    for i in 0..number {
        let mut next_state = now_state.clone();
        next_state.transition();
        let next_score = next_state.get_score(false);

        let temp = start_temp + (end_temp - start_temp) * (i as f64 / number as f64);
        let probability = (-(next_score as f64 - now_score as f64) / temp).exp(); // 確率probで遷移する

        let is_force_next = probability > rng.gen_range(0.0..1.0);
        if next_score > now_score || is_force_next {
            now_score = next_score;
            now_state = next_state.clone();
        }

        if next_score > best_score {
            best_score = next_score;
            best_state = next_state.clone();
        }
    }

    best_state
}


struct StringAIPair {
    name: String,
    ai: AIFunction,
}

// ゲームを1回プレイしてゲーム状況を表示する
// fn play_game(ai: &StringAIPair, seed: usize) {
//     let mut state = AutoMoveMazeState::new(Some(seed));
//     state = (ai.ai)(&state);
//     println!("{}", state.to_string());
//     let score = state.get_score(true);
//     println!("Score of {}: {}", ai.name, score);
// }

// ゲームをgame_number回プレイして平均スコアを表示する
fn test_ai_score(ai: &StringAIPair, game_number: usize) {
    let mut score_mean = 0.0;

    for i in 0..game_number {
        let mut state = AutoMoveMazeState::new(Some(i));
        state = (ai.ai)(&state);

        let score = state.get_score(false);
        score_mean += score as f64;
    }

    score_mean /= game_number as f64;
    println!("Score of {}: {}", ai.name, score_mean);
}

#[allow(dead_code)]
pub fn main() {
    let ais = [
        StringAIPair {
            name: "hill_climb".to_string(),
            ai: |state| {hill_climb(state, 10000)},
        },
        StringAIPair {
            name: "simulated_annealing".to_string(),
            ai: |state| {simulated_annealing(state, 10000, 500.0, 10.0)},
        }
    ];
    for ai in ais {
        test_ai_score(&ai, 100); // 盤面生成シードを0に設定してプレイする。
    }
}
