use std::{collections::HashSet, io};

use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

#[derive(Clone, Debug, PartialEq)]
enum Tile {
    Flagged,
    Concealed,
    Open(usize),
}

#[derive(Clone, Debug)]
enum InputAction {
    Flag(usize),
    Open(usize),
    Exit,
}

#[derive(Clone, Debug)]
enum InputError {
    ParseError,
    InvalidCoords((usize, usize)),
}

fn main() {
    let row_count = 10_usize;
    let col_count = 5_usize;
    let mine_count = 1;

    let mut board = vec![Tile::Concealed; col_count*row_count];
    let mut mine_indeces = HashSet::new();

    loop {
        clearscreen::clear().expect("Failed to print to console");
        print_board(&board, row_count, col_count);
        print_menu();

        let action = match request_input(row_count, col_count) {
            Ok(res) => res,
            Err(error) => {
                print_parser_error(error);
                continue;
            }
        };

        match action {
            InputAction::Exit => return,
            InputAction::Flag(tile_index) => flag_tile(&mut board, tile_index),
            InputAction::Open(tile_index) => {

                if mine_indeces.capacity() == 0 {
                    mine_indeces =
                        generate_mine_positions(mine_count, tile_index, row_count, col_count);
                }

                if mine_indeces.contains(&tile_index) {
                    println!("You lost!");
                    return;
                }

                reveal_tiles(&mut board, tile_index, &mine_indeces, row_count, col_count);
            }
        }
    }
}

fn reveal_tiles(
    board: &mut [Tile],
    tile_idx: usize,
    mine_indeces: &HashSet<usize>,
    row_count: usize,
    col_count: usize,
) {
    reveal_recoursively(board, mine_indeces, &HashSet::from_iter(vec![tile_idx]), row_count, col_count);
}

fn reveal_recoursively(
    board: &mut [Tile],
    mine_indeces: &HashSet<usize>,
    tiles_to_reveal: &HashSet<usize>,
    row_count: usize,
    col_count: usize,
) {
    if tiles_to_reveal.is_empty() {
        return;
    }

    let mut neighbours_to_reveal = HashSet::new();
    for &tile_idx in tiles_to_reveal {

        if board[tile_idx] == Tile::Concealed {
            let mine_count =
                count_neighbouring_mines(tile_idx, mine_indeces, row_count, col_count);
            board[tile_idx] = Tile::Open(mine_count);

            if mine_count == 0 {
                neighbours_to_reveal.extend(
                    get_neighbouring_indices(tile_idx, row_count, col_count)
                        .into_iter()
                        .filter(|&idx| board[idx] == Tile::Concealed),
                );
            }
        }

        reveal_recoursively(board, mine_indeces, &neighbours_to_reveal, row_count, col_count);
    }
}

fn flag_tile(board: &mut [Tile], tile_idx: usize) {
    board[tile_idx] = match board[tile_idx] {
        Tile::Flagged => Tile::Concealed,
        Tile::Concealed => Tile::Flagged,
        Tile::Open(count) => Tile::Open(count),
    }
}

fn print_board(board: &[Tile], row_count: usize, col_count: usize) {
    let res = board
        .chunks(col_count)
        .map(|row| {
            row.iter()
                .map(|tile| match tile {
                    Tile::Flagged => "F".to_string(),
                    Tile::Open(count) => count.to_string(),
                    Tile::Concealed => "#".to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n");

    println!("{}", res);
}

fn print_menu() {
    println!("\n");
    println!("Type 'x <row> <col>' to open a tile");
    println!("Type 'f <row> <col>' to set a flag");
    println!("Type 'exit' to exit");
    println!("\n");
}

fn request_input(row_size: usize, col_size: usize) -> Result<InputAction, InputError> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input = input.trim().to_lowercase();

    if input == "exit" {
        return Ok(InputAction::Exit);
    }

    let parts = input.split(' ').collect::<Vec<_>>();

    if parts.len() != 3 {
        return Err(InputError::ParseError);
    }

    let row = parts[1].parse::<usize>().or(Err(InputError::ParseError))?;
    let col = parts[2].parse::<usize>().or(Err(InputError::ParseError))?;

    if row >= row_size || col >= col_size {
        return Err(InputError::InvalidCoords((row, col)));
    }

    match parts[0] {
        "f" => Ok(InputAction::Flag(row * row_size + col)),
        "x" => Ok(InputAction::Open(row * row_size + col)),
        _ => Err(InputError::ParseError),
    }
}

fn print_parser_error(error: InputError) {
    println!();

    match error {
        InputError::ParseError => println!("Failed to parse the input. Please try again"),
        InputError::InvalidCoords(coords) => println!("The coords {:?} are invalid", coords),
    };

    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read input");
}

fn generate_mine_positions(
    count: usize,
    index_to_avoid: usize,
    row_count: usize,
    col_count: usize,
) -> HashSet<usize> {
    let mut rng = rand::thread_rng();

    let indices_to_avoid = get_neighbouring_indices(index_to_avoid, row_count, col_count);

    let indices = (0..row_count * col_count)
        .filter(|idx| !indices_to_avoid.contains(idx) || *idx != index_to_avoid)
        .choose_multiple(&mut rng, count);

    HashSet::from_iter(indices)
}

fn get_neighbouring_indices(index: usize, row_count: usize, col_count: usize) -> HashSet<usize> {
    let max_index = row_count * col_count -1;
    HashSet::from_iter(vec![
        index.saturating_sub(col_count + 1),
        index.saturating_sub(col_count),
        index.saturating_sub(col_count - 1),
        index.saturating_sub(1),
        index.saturating_add(1).clamp(0, max_index),
        index.saturating_add(col_count - 1).clamp(0, max_index),
        index.saturating_add(col_count).clamp(0, max_index),
        index.saturating_add(col_count + 1).clamp(0, max_index),
    ])
}

fn count_neighbouring_mines(
    tile_idx: usize,
    bomb_indices: &HashSet<usize>,
    row_count: usize,
    col_count: usize,
) -> usize {
    get_neighbouring_indices(tile_idx, row_count, col_count)
        .iter()
        .filter(|idx| bomb_indices.contains(idx))
        .count()
}
