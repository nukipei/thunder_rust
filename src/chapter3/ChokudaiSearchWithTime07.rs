#![allow(non_snake_case)]

use rand::{Rng, SeedableRng, rngs, thread_rng};
use std::collections::BinaryHeap;
use std::time::Instant;

// 時間を管理する構造体
struct TimeKeeper {
    start_time: Instant,
    time_threshold: usize,
}

impl TimeKeeper {
    // 時間制限をミリ秒単位で指定してインスタンスをつくる。
    fn new(time_threshold: usize) -> Self {
        TimeKeeper {
            start_time: Instant::now(),
            time_threshold,
        }
    }

    // インスタンス生成した時から指定した時間制限を超過したか判定する。
    fn is_time_over(&self) -> bool {
        let elapsed_time = self.start_time.elapsed().as_millis() as usize;
        elapsed_time >= self.time_threshold
    }
}

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
const H: usize = 30;
const W: usize = 30;
// ゲーム終了ターン
const END_TURN: usize = 100;

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
    first_action: i32,
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
        let first_action = -1;  // 探索木のルートノードで最初に選択した行動

        MazeState {
            character,
            points,
            turn,
            game_score,
            evaluated_score,
            first_action,
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

// 探索時のソート用に評価を比較する
impl Ord for MazeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluated_score.cmp(&other.evaluated_score)
    }
}

impl PartialOrd for MazeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MazeState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_score == other.evaluated_score
    }
}

impl Eq for MazeState {}

// ビーム1本あたりのビームの幅と深さ、本数を指定してchokudaiサーチで行動を決定する
fn chokudai_search_action_wirh_time_threshold(state: &MazeState, beam_width: usize, beam_depth: usize, time_threshold: usize) -> usize {
    let time_keeper = TimeKeeper::new(time_threshold);

    let mut beam: Vec<BinaryHeap<MazeState>> = vec![BinaryHeap::new(); beam_depth + 1];
    // let mut beam: Vec<BinaryHeap<MazeState>> = Vec::with_capacity(beam_depth + 1);
    // beam.extend((0..=beam_depth).map(|_| BinaryHeap::new()));

    beam[0].push(state.clone());

    loop {
        for t in 0..beam_depth {
            let mut now_beam = beam[t].clone();
            let mut next_beam = beam[t + 1].clone();

            for _ in 0..beam_width {
                if now_beam.is_empty() {
                    break;
                }

                if now_beam.peek().unwrap().clone().is_done() {
                    break;
                }
                let now_state = now_beam.pop().unwrap();

                let legal_actions = now_state.legal_actions();

                for &action in &legal_actions {
                    let mut next_state = now_state.clone();
                    next_state.advance(action);
                    next_state.evaluate_score();

                    if t == 0 {
                        next_state.first_action = action as i32;
                    }

                    next_beam.push(next_state);
                }
            }

            beam[t] = now_beam;
            beam[t + 1] = next_beam;
        }
        if time_keeper.is_time_over() {
            break;
        }
    }

    for t in (0..=beam_depth).rev() {
        let now_beam = &beam[t];
        if let Some(best_state) = now_beam.peek() {
            return best_state.first_action as usize;
        }
    }

    0 // ここには来ないはず
}

// ゲームをgame_number回プレイして平均スコアを表示する
fn test_ai_score(game_number: usize) {
    let mut score_mean = 0.0;

    for _ in 0..game_number {
        let mut state = MazeState::new(None);

        // let mut c = 1;
        while !state.is_done() {
            let action = chokudai_search_action_wirh_time_threshold(&state, 1, END_TURN, 10);
            state.advance(action);
            // println!("{}, {}, {}", c, action, state.game_score);
            // c += 1;
        }

        let score = state.game_score;
        // println!("score:\t{}", score);
        score_mean += score as f64;
    }

    score_mean /= game_number as f64;
    println!("Score:\t{}", score_mean);
}


#[allow(dead_code)]
pub fn main() {
    let start_time = Instant::now();
    test_ai_score(10);

    // 経過時間を秒で表示する
    let elapsed_time = start_time.elapsed().as_secs_f64();
    println!("elapsed_time:\t{}", elapsed_time);
}