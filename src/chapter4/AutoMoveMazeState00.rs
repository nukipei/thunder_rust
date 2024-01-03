#![allow(non_snake_case)]

use rand::{Rng, SeedableRng, rngs, thread_rng};

const H: usize = 5;        // 迷路の高さ
const W: usize = 5;        // 迷路の幅
const END_TURN: usize = 5;  // ゲーム終了ターン
const CHARACTER_N: usize = 3; // キャラクターの数

type ScoreType = i64;
const INF: ScoreType = 1000000000;

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
}

#[allow(non_upper_case_globals)]
const dy: [isize; 4] = [0, 0, 1, -1];

#[allow(non_upper_case_globals)]
const dx: [isize; 4] = [1, -1, 0, 0];

fn random_action(state: &AutoMoveMazeState) -> AutoMoveMazeState {
    let mut now_state = state.clone();
    // let mut rng = rand::thread_rng();
    let mut rng: rngs::StdRng = SeedableRng::seed_from_u64(0 as u64);

    for character_id in 0..CHARACTER_N {
        let y = rng.gen_range(0..H);
        let x = rng.gen_range(0..W);

        now_state.set_character(character_id, y, x);
    }

    now_state
}

type AIFunction = fn(&AutoMoveMazeState) -> AutoMoveMazeState;

struct StringAIPair {
    name: String,
    ai: AIFunction,
}

// ゲームを1回プレイしてゲーム状況を表示する
fn play_game(ai: &StringAIPair, seed: usize) {
    let mut state = AutoMoveMazeState::new(Some(seed));
    state = (ai.ai)(&state);
    println!("{}", state.to_string());
    let score = state.get_score(true);
    println!("Score of {}: {}", ai.name, score);
}

#[allow(dead_code)]
pub fn main() {
    let ai = StringAIPair {
        name: "random_action".to_string(),
        ai: random_action,
    };
    play_game(&ai, 0); // 盤面生成シードを0に設定してプレイする。
}
