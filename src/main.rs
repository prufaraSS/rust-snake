use std::{
    io::{stdout,Write},
    fs,
    fs::OpenOptions,
    time::{SystemTime,Duration},
    collections::VecDeque
};

mod general;
use crate::general::{error_handling::*,graphics::*,input::*};

use crossterm::{
    execute, queue,
    terminal::{
        DisableLineWrap,
        EnableLineWrap,
        SetTitle,
        SetSize,
        EnterAlternateScreen,
        LeaveAlternateScreen,
        enable_raw_mode,
        disable_raw_mode,
        Clear,
        ClearType,
        size
    },
    event::{EnableMouseCapture,DisableMouseCapture,read,Event},
    cursor::{Hide,Show,MoveTo},
    style::{Print,Stylize,SetForegroundColor,SetBackgroundColor,Color}
};

use rand::{thread_rng,Rng};

enum Screen {
    MainMenu,
    Edit,
    Game
}

struct Pos {
    x:u16,
    y:u16
}

struct Button {
    x:u16,
    y:u16,
    width:u16,
    height:u16,
    return_code:u8
}

pub enum Direction {
    Up,
    Right,
    Left,
    Down
}

trait DirectionFunctionality {
    fn copy(&self) -> Direction;
    fn is_opposite_of(&self, direction:&Direction) -> bool;
}
impl DirectionFunctionality for Direction {
    fn copy(&self) -> Direction {
        match self {
            Direction::Right => Direction::Right,
            Direction::Left => Direction::Left,
            Direction::Up => Direction::Up,
            Direction::Down => Direction::Down
        }
    }
    fn is_opposite_of(&self, direction: &Direction) -> bool {
        match self {
            Direction::Right => if let Direction::Left = direction {true} else {false},
            Direction::Left => if let Direction::Right = direction {true} else {false},
            Direction::Up => if let Direction::Down = direction {true} else {false},
            Direction::Down => if let Direction::Up = direction {true} else {false}
        }
    }
}


pub struct Snake {
    pos:Pos,
    body: VecDeque<Pos>,
    direction:Direction,
    alive:bool,
    prev_move:Direction,
    last_input:Direction
}

trait SnakeFunctionality {
    fn new() -> Snake;
    fn is_in_point(&self,x:u16,y:u16) -> bool;
}

impl SnakeFunctionality for Snake {
    fn new() -> Snake {
        Snake {
            pos:Pos{
                x:SNAKE_SPAWN_POS_X+2,
                y:SNAKE_SPAWN_POS_Y
            },
            body:VecDeque::from([
                Pos{
                    x:SNAKE_SPAWN_POS_X,
                    y:SNAKE_SPAWN_POS_Y
                },
                Pos{
                    x:SNAKE_SPAWN_POS_X+1,
                    y:SNAKE_SPAWN_POS_Y
                }
            ]),
            direction:Direction::Right,
            alive:true,
            prev_move:Direction::Right,
            last_input:Direction::Right
        }
    }
    fn is_in_point(&self,x:u16,y:u16) -> bool {
        if (self.pos.y == y) && (self.pos.x == x) {return true}
        let mut iter = self.body.iter();
        while let Some(body) = iter.next() {
            if (body.y == y) && (body.x == x) {return true}
        }
        false
    }
}

struct Fruit {
    pos:Pos
}

trait FruitFunctionality {
    fn new(map:&[[bool;GAME_FIELD_SIZE];GAME_FIELD_SIZE]) -> Fruit;
    fn respawn(&mut self,map:&[[bool;GAME_FIELD_SIZE];GAME_FIELD_SIZE],snake:&Snake);
}

impl FruitFunctionality for Fruit {
    fn new(map:&[[bool;GAME_FIELD_SIZE];GAME_FIELD_SIZE]) -> Fruit {
        let mut f = Fruit {
            pos: Pos{x:0,y:0} 
        };
        f.respawn(map,&mut Snake::new());
        f
    }
    fn respawn(&mut self,map:&[[bool;GAME_FIELD_SIZE];GAME_FIELD_SIZE],snake:&Snake) {
        //damn i'm always getting suprised how large simple code can become in rust
        let mut rng = thread_rng();
        let mut lines = vec![];
        let mut num = 0u16;
        for line in map {
            let mut x = 0u16;
            for val in line { // filtering out all lines with no free space
                if val == &false &&
                    snake.is_in_point(x,num) == false {
                        lines.push(num);
                        break;
                }
                x += 1;
            }
            num += 1;
        } 
        let size = lines.iter().count();
        if size == 0 {
            self.pos = Pos {x: 0, y: 0};
            return
        }
        let line = rng.gen_range(0..size); // selecting line randomly
        let mut points = vec![];
        let mut num = 0u16;
        for row in map[lines[line] as usize] {
            if row == false && 
                snake.is_in_point(num, lines[line]) == false { // filtering out all walls from selected line
                    points.push(num)
            }
            num += 1;
        } 

        let size = points.iter().count();
        let point = rng.gen_range(0..size); // selecting point in line randomly
        self.pos = Pos {
            x: points[point],
            y: lines[line]
        }
    }
}


const FILE_TITLE:&str = "title.txt";
const FILE_BUTTONS:&str = "buttons.txt";
const FILE_TITLECOLORS:&str = "titlecolors.txt";
const FILE_EDITOR:&str = "editor.txt";
const FILE_GAME:&str = "game.txt";
const FILE_MAP:&str = "map.txt";
const MENU_TICK:u64 = 250; //lower = faster
const GAME_TICK :u64 = 250; //lower = faster
const NONE:u8 = 0; //"no button selected" constant
const BUTTON_PLAY:u8 = 1;
const BUTTON_EXIT:u8 = 2;
const BUTTON_EDIT:u8 = 3;
const GLOBAL_OFFSET_X:u16 = 1;
const GLOBAL_OFFSET_Y:u16 = 1;
const BUTTONS_POS :Pos = Pos {
    x:GLOBAL_OFFSET_X,
    y:13
};
const EDIT_RESET_BUTTON:Button = Button {
    x: EDIT_HINT_OFFSET_X,
    y: EDIT_HINT_OFFSET_Y + 8,
    width: 6,
    height: 2,
    return_code: 1
};
const MAINMENU_BUTTONS:[Button;3] = [
    Button {
        x: BUTTONS_POS.x,
        y: BUTTONS_POS.y,
        width: 10,
        height: 3,
        return_code: BUTTON_PLAY
    },
    Button {
        x: BUTTONS_POS.x,
        y: BUTTONS_POS.y+5,
        width: 10,
        height: 3,
        return_code: BUTTON_EDIT
    }, 
    Button {
        x: BUTTONS_POS.x,
        y: BUTTONS_POS.y+10,
        width: 10,
        height: 3,
        return_code: BUTTON_EXIT
    }
];
const GAME_FIELD_OFFSET_X:u16 = GLOBAL_OFFSET_X;
const GAME_FIELD_OFFSET_Y:u16 = 3;
const EDIT_HINT_OFFSET_X:u16 = 30;
const EDIT_HINT_OFFSET_Y:u16 = GLOBAL_OFFSET_Y;
const EDIT_HINT_SIZE_X:u16 = 52;
const EDIT_HINT_SIZE_Y:u16 = 11;
const SNAKE_SPAWN_POS_X:u16 = 2;
const SNAKE_SPAWN_POS_Y:u16 = 2;
const GAME_TIME_OFFSET:u16 = GLOBAL_OFFSET_X + 34;
const GAME_SCORE_OFFSET:u16 = GLOBAL_OFFSET_Y + 7;
const GAME_FIELD_SIZE:usize = 20;
const APPLE:char = '¤'; //this sign is supportable with all fonts
const COLOR_GRAY :Color = Color::Rgb{r:40,g:40,b:40};
const COLOR_RESET :Color = Color::Reset;
const COLOR_WHITE :Color = Color::White;
const COLOR_BLUE :Color = Color::Blue;
const COLOR_YELLOW :Color = Color::Yellow;
const COLOR_RED :Color = Color::Red;
const COLOR_GREEN :Color = Color::Green;
//edit screen hint is the widest part and main menu buttons are the tallest part
const SCREEN_MIN_SIZE_X:u16 = EDIT_HINT_OFFSET_X + EDIT_HINT_SIZE_X;
const SCREEN_MIN_SIZE_Y:u16 = BUTTONS_POS.y+16;

fn draw(cursor:&Cursor,color:Color) {
    queue!(
        stdout(),
        MoveTo(cursor.x,cursor.y),
        Print(" ".on(color)),
    ).handle();
}

fn back_to_main_menu(buttons_ascii:&String) {
    queue!(stdout(),Clear(ClearType::All)).handle();
    draw_simple_ascii_picture(buttons_ascii,BUTTONS_POS.x,BUTTONS_POS.y);
    stdout().flush().handle();
}

fn read_file(s:&str) -> String {
    fs::read_to_string(s).handle_read(s)
}

fn get_hover(posx:u16,posy:u16,button:&Button) -> u8 {
    if posx.checked_sub(button.x).unwrap_or(u16::MAX) <= button.width &&
        posy.checked_sub(button.y).unwrap_or(u16::MAX) <= button.height {
            return button.return_code
    }
    NONE
}

fn draw_map(map:&[[bool;GAME_FIELD_SIZE];GAME_FIELD_SIZE],offsetx:u16,offsety:u16) {
    let mut y = 0;
    for line in map {
        queue!(
            stdout(),
            MoveTo(
                offsetx,
                offsety + y
            )
        ).handle();
        for row in line {
            queue!(
                stdout(),
                Print(" ".on(
                    if row == &true {
                        COLOR_YELLOW
                    } else {
                        COLOR_GRAY
                    }
                ))
            ).handle()
            
        }
        y += 1;
    }
}

fn reset_map(map: &mut [[bool;GAME_FIELD_SIZE];GAME_FIELD_SIZE]) {
    for y in 0..(GAME_FIELD_SIZE) {
        if y == 0 || y == (GAME_FIELD_SIZE-1) {map[y] = [true;GAME_FIELD_SIZE];}
        else {
            map[y][0] = true;
            map[y][(GAME_FIELD_SIZE-1)] = true;
        }
    }
}

pub fn free_window(w:u16,h:u16) {
    disable_raw_mode().handle();
    if w+h != 0 {
        queue!(
            stdout(),
            SetSize(w,h)
        ).handle();
    }
    execute!(
        stdout(),
        EnableLineWrap,
        DisableMouseCapture,
        LeaveAlternateScreen,
        Show
    ).handle();
}

fn max(first_num:u16,second_num:u16) -> u16 {
    if first_num < second_num {second_num}
    else {first_num}
}

fn main() {
    
    //Setup window
    let (term_old_w,term_old_h) = size().unwrap();
    execute!(
        stdout(),
        DisableLineWrap,
        SetTitle("Snake"),
        SetSize(
            max(term_old_w,SCREEN_MIN_SIZE_X),
            max(term_old_h,SCREEN_MIN_SIZE_Y)
        ),
        EnterAlternateScreen,
        EnableMouseCapture,
        Hide
    ).handle();
    enable_raw_mode().handle();

    let mut form = Screen::MainMenu;

    //reading files
    let title = read_file(FILE_TITLE); //title ascii picture
    let buttons_ascii = read_file(FILE_BUTTONS); //main menu buttons ascii
    let title_colors = read_file(FILE_TITLECOLORS); //colors for title
    let game_field = read_file(FILE_GAME); //game scene
    let edit_screen = read_file(FILE_EDITOR); //field edit scene
    let map_string = fs::read_to_string("map.txt"); // saved map 
    
    let title_colors_size = title_colors.lines().count();
    let mut title_colors_iter:usize = 0;
    //converting pool of strings to pool of colors :pogchamp:
    let title_colors = title_colors.lines().map( 
        |x|
        match Color::parse_ansi( &format!("2;{}",x) ) {
            Some(color) => color,
            None => COLOR_WHITE
        }
    ).collect::<Vec<Color>>();

    let mut tick = SystemTime::now();

    let mut cursor = Cursor { //cursor on main menu
        x: 0,
        y: 0,
        hover: NONE
    };

    let mut map = [[false;GAME_FIELD_SIZE];GAME_FIELD_SIZE];
    
    if let Ok(s) = map_string { //if life gives you files - read them
        let mut y = 0;
        let mut x = 0;
        let mut file_map_lines = s.lines();
        while let Some(line) = file_map_lines.next() {
            if y >= GAME_FIELD_SIZE {break;}
            let mut chars = line.chars();
            while let Some(ch) = chars.next() {
                if x >= GAME_FIELD_SIZE {break;}
                map[y][x] = if ch == '1' {true} else {false};
                x += 1;
            }
            x = 0;
            y += 1;
        }
    } else {
        reset_map(&mut map);
    }

    let mut snake = Snake::new();
    let mut fruit = Fruit::new(&map);
    let mut score = 0u16;
    let mut time = SystemTime::now();
    
    draw_simple_ascii_picture(&buttons_ascii,BUTTONS_POS.x,BUTTONS_POS.y);
    stdout().flush().handle();

    //game loop
    loop {
        match form {
            Screen::MainMenu => {
                let menu_elapsed = tick.elapsed().unwrap().as_millis();
                if menu_elapsed >= MENU_TICK as u128 {
                    title_colors_iter += 1;
                    if title_colors_iter >= title_colors_size {
                        title_colors_iter = 0;
                    }
                    tick = SystemTime::now();
                }
                //got rid of derefencing, but at what cost?
                queue!(stdout(),SetForegroundColor(title_colors[title_colors_iter])).handle();
                draw_simple_ascii_picture(
                    &title,
                    GLOBAL_OFFSET_X,
                    GLOBAL_OFFSET_Y
                );
                queue!(stdout(),SetForegroundColor(COLOR_RESET)).handle();
                stdout().flush().handle();

                draw(&cursor,COLOR_RESET);
                
                if cursor.hover != NONE {
                    draw_simple_ascii_picture(&buttons_ascii,BUTTONS_POS.x,BUTTONS_POS.y);
                }
                
                let input_result = cursor_input(
                    &mut cursor,Duration::from_millis(
                        GAME_TICK.checked_sub(menu_elapsed as u64).unwrap_or(0)
                    )
                );
                
                draw(
                    &cursor,
                    if cursor.hover == NONE {
                        COLOR_WHITE
                    } else {
                        COLOR_BLUE
                    }
                );

                stdout().flush().handle();

                match input_result {
                    InputResult::Abort => break,
                    InputResult::Click => match cursor.hover {
                        BUTTON_PLAY => {
                            form = Screen::Game;
                            execute!(
                                stdout(),
                                Clear(ClearType::All)
                            ).handle();
                            draw_simple_ascii_picture(&game_field,1,1);
                            draw_map(&map,GAME_FIELD_OFFSET_X,GAME_FIELD_OFFSET_Y);
                            stdout().flush().handle();
                            snake = Snake::new();
                            let tail = snake.body.front().unwrap();
                            execute!(
                                stdout(),
                                SetBackgroundColor(COLOR_GRAY),
                                MoveTo(
                                    snake.pos.x + GAME_FIELD_OFFSET_X,
                                    snake.pos.y + GAME_FIELD_OFFSET_Y
                                ),
                                Print('►'.green()),
                                MoveTo(
                                    tail.x + GAME_FIELD_OFFSET_X,
                                    tail.y + GAME_FIELD_OFFSET_Y
                                ),
                                Print('═'.green()),
                            ).handle();
                            for body in &snake.body {
                                queue!(
                                    stdout(),
                                    SetBackgroundColor(COLOR_GRAY),
                                    MoveTo(
                                        body.x + GAME_FIELD_OFFSET_X,
                                        body.y + GAME_FIELD_OFFSET_Y
                                    ),
                                    Print('═'.green())
                                ).handle();
                            }
                            fruit.respawn(&map,&snake);
                            queue!(
                                stdout(),
                                MoveTo(
                                    fruit.pos.x + GAME_FIELD_OFFSET_X,
                                    fruit.pos.y + GAME_FIELD_OFFSET_Y
                                ),
                                Print(APPLE.red()),
                                SetBackgroundColor(COLOR_RESET),
                                MoveTo(GAME_SCORE_OFFSET,GLOBAL_OFFSET_Y),
                                Print(score),
                                MoveTo(GAME_TIME_OFFSET,GLOBAL_OFFSET_Y),
                                Print(0)
                            ).handle();
                            stdout().flush().handle();
                            loop {
                                if let Event::Key(_) = read().expect("Can't detect pressed key") {
                                    execute!(
                                        stdout(),
                                        MoveTo(27,3),
                                        Clear(ClearType::UntilNewLine)
                                    ).handle();
                                    break;
                                }
                            }
                            tick = SystemTime::now();
                            time = SystemTime::now();
                        },
                        BUTTON_EXIT => break,
                        BUTTON_EDIT => {
                            form = Screen::Edit;
                            queue!(
                                stdout(),
                                Clear(ClearType::All)
                            ).handle();
                            draw_simple_ascii_picture(
                                &edit_screen,
                                EDIT_HINT_OFFSET_X,
                                EDIT_HINT_OFFSET_Y
                            );

                            draw_map(&map,GLOBAL_OFFSET_X,GLOBAL_OFFSET_Y);
                            queue!(
                                stdout(),
                                MoveTo(
                                    GLOBAL_OFFSET_X + SNAKE_SPAWN_POS_X,
                                    GLOBAL_OFFSET_Y + SNAKE_SPAWN_POS_Y,
                                ),
                                Print("   ".on_red())
                            ).handle();
                            stdout().flush().handle();
                        }
                        _ => ()
                    },
                    _ => ()
                }

                cursor.hover = NONE;
                for button in &MAINMENU_BUTTONS {
                    let code = get_hover(cursor.x,cursor.y,&button);
                    if code != NONE {
                        cursor.hover = code;
                        break
                    }
                }
            },
            Screen::Game => {
                let elapsed = tick.elapsed().unwrap().as_millis();
                let input = game_input(
                    &mut snake,
                    Duration::from_millis(
                        GAME_TICK.checked_sub(elapsed as u64).unwrap_or(0)
                    )
                );
                if elapsed >= GAME_TICK as u128 {
                    tick = SystemTime::now();
                    if snake.alive == false {
                        execute!(
                            stdout(),
                            SetForegroundColor(COLOR_RESET),
                            MoveTo(29,3),
                            Print("You died! Press ESC to return back to menu".red())
                        ).handle();
                        continue
                    }
                    snake.body.push_back(
                        Pos {
                            x:snake.pos.x,
                            y:snake.pos.y
                        }
                    );
                    //makes much easier to turn 180 degrees
                    if snake.last_input.is_opposite_of(&snake.prev_move) == false {
                        snake.direction = snake.last_input.copy()
                    };
                    snake.pos.x = match snake.direction { //snake movement x
                        Direction::Right => {
                            if snake.pos.x < (GAME_FIELD_SIZE-1) as u16 {
                                snake.pos.x + 1
                            } else {
                                0
                            }
                        },
                        Direction::Left => {
                            snake.pos.x
                                .checked_sub(1)
                                .unwrap_or((GAME_FIELD_SIZE-1) as u16)
                        },
                        _ => snake.pos.x
                    };
                    snake.pos.y = match snake.direction { //snake movement y
                        Direction::Up => {
                            snake.pos.y
                                .checked_sub(1)
                                .unwrap_or((GAME_FIELD_SIZE-1) as u16)
                        },
                        Direction::Down => {
                            if snake.pos.y < (GAME_FIELD_SIZE-1) as u16 {
                                snake.pos.y + 1
                            } else {
                                0
                            }
                        },
                        _ => snake.pos.y
                    };
                    let tail = snake.body.front().unwrap();
                    queue!(
                        stdout(),
                        SetBackgroundColor(COLOR_GRAY),
                        MoveTo(
                            tail.x + GAME_FIELD_OFFSET_X,
                            tail.y + GAME_FIELD_OFFSET_Y
                        ),
                        Print(' '),
                        MoveTo(
                            snake.pos.x + GAME_FIELD_OFFSET_X,
                            snake.pos.y + GAME_FIELD_OFFSET_Y
                        ),
                        SetForegroundColor(COLOR_GREEN),
                        Print(
                            match snake.direction {
                                Direction::Right => '►',
                                Direction::Left => '◄',
                                Direction::Down => '▼',
                                Direction::Up => '▲'
                            }
                        ),
                        SetForegroundColor(COLOR_RESET)
                    ).handle();
                    
                    queue!(
                        stdout(),
                        MoveTo(
                            snake.body.back().unwrap().x + GAME_FIELD_OFFSET_X,
                            snake.body.back().unwrap().y + GAME_FIELD_OFFSET_Y
                        ),
                        SetForegroundColor(COLOR_GREEN),
                        match snake.prev_move { //graphics of snake rotation
                            Direction::Right => { //formula: invert prev_move and copy cur dir
                                match snake.direction {
                                    Direction::Right | Direction::Left => Print('═'),
                                    Direction::Up => Print('╝'),
                                    Direction::Down => Print('╗')
                                }
                            },
                            Direction::Left => {
                                match snake.direction {
                                    Direction::Right | Direction::Left => Print('═'),
                                    Direction::Up => Print('╚'),
                                    Direction::Down => Print('╔')
                                }
                            },
                            Direction::Up => {
                                match snake.direction {
                                    Direction::Right => Print('╔'),
                                    Direction::Left => Print('╗'),
                                    Direction::Up | Direction::Down => Print('║')
                                }
                            },
                            Direction::Down => {
                                match snake.direction {
                                    Direction::Right => Print('╚'),
                                    Direction::Left => Print('╝'),
                                    Direction::Up | Direction::Down => Print('║')
                                }
                            } 
                        },
                        SetBackgroundColor(COLOR_RESET),
                        SetForegroundColor(COLOR_RESET),
                        MoveTo(GAME_TIME_OFFSET,GLOBAL_OFFSET_Y),
                        Print(time.elapsed().unwrap().as_secs())
                    ).handle();
                    if snake.pos.x == fruit.pos.x && snake.pos.y == fruit.pos.y {
                        fruit.respawn(&map,&snake);
                        score += 1;
                        queue!(
                            stdout(),
                            MoveTo(GAME_SCORE_OFFSET,GLOBAL_OFFSET_Y),
                            Print(score),
                            MoveTo(fruit.pos.x+GAME_FIELD_OFFSET_X,fruit.pos.y+GAME_FIELD_OFFSET_Y),
                            SetBackgroundColor(COLOR_GRAY),
                            Print(APPLE.red()),
                        ).handle();
                    } else {
                        snake.body.pop_front();
                    }
                    stdout().flush().handle();
                    snake.prev_move = snake.direction.copy();
                    let body = &mut snake.body;
                    let pos = &snake.pos;
                    for part in body {
                        if part.x == pos.x && part.y == pos.y {
                            snake.alive = false;
                            continue;
                        }
                    }
                    if map[snake.pos.y as usize][snake.pos.x as usize] == true {
                        snake.alive = false;
                        continue;
                    }
                }
                if let InputResult::Abort = input {
                    form = Screen::MainMenu;
                    score = 0;
                    back_to_main_menu(&buttons_ascii);
                }
            },
            Screen::Edit => {
                let parsed_cursor_position = Pos {
                    x: cursor.x.checked_sub(GLOBAL_OFFSET_X).unwrap_or(GAME_FIELD_SIZE as u16),
                    y: cursor.y.checked_sub(GLOBAL_OFFSET_Y).unwrap_or(GAME_FIELD_SIZE as u16)
                };

                draw( //draw wall/non-wall under old cursor position
                    &cursor,
                    if (parsed_cursor_position.x < GAME_FIELD_SIZE as u16) && 
                        (parsed_cursor_position.y < GAME_FIELD_SIZE as u16)
                    {
                        if map
                            [parsed_cursor_position.y as usize]
                            [parsed_cursor_position.x as usize] == true 
                        {
                            COLOR_YELLOW
                        } else {
                            if (parsed_cursor_position.x >= SNAKE_SPAWN_POS_X) &&
                                (parsed_cursor_position.x <= SNAKE_SPAWN_POS_X+2) &&
                                (parsed_cursor_position.y == SNAKE_SPAWN_POS_Y) == true
                            {
                                COLOR_RED
                            } else {
                                COLOR_GRAY
                            }
                        }
                    } else {
                        COLOR_RESET
                    }
                );

                if (cursor.x >= EDIT_HINT_OFFSET_X) && 
                   (cursor.y >= EDIT_HINT_OFFSET_Y) &&
                   (cursor.x <= EDIT_HINT_OFFSET_X + EDIT_HINT_SIZE_X) &&
                   (cursor.y <= EDIT_HINT_OFFSET_Y + EDIT_HINT_SIZE_Y)
                {
                    draw_simple_ascii_picture(
                        &edit_screen, 
                        EDIT_HINT_OFFSET_X, 
                        EDIT_HINT_OFFSET_Y
                    )
                }

                let input_result = cursor_input(
                    &mut cursor,
                    Duration::MAX
                );

                let parsed_cursor_new_position = Pos {
                    x: cursor.x.checked_sub(GLOBAL_OFFSET_X).unwrap_or(GAME_FIELD_SIZE as u16),
                    y: cursor.y.checked_sub(GLOBAL_OFFSET_Y).unwrap_or(GAME_FIELD_SIZE as u16)
                };
                let valid = (parsed_cursor_new_position.x < GAME_FIELD_SIZE as u16) &&
                            (parsed_cursor_new_position.y < GAME_FIELD_SIZE as u16); //is cursor on map
                let snake_rewrite =
                    (parsed_cursor_new_position.x >= SNAKE_SPAWN_POS_X) &&
                    (parsed_cursor_new_position.x <= SNAKE_SPAWN_POS_X + 2) && //is cursor at snake spawn pos
                    (parsed_cursor_new_position.y == SNAKE_SPAWN_POS_Y);
                if let InputResult::Draw = input_result {
                    if valid == true && snake_rewrite == false {
                        if (parsed_cursor_new_position.x != parsed_cursor_position.x) ||
                           (parsed_cursor_new_position.y != parsed_cursor_position.y)
                        {
                            map
                                [parsed_cursor_new_position.y as usize]
                                [parsed_cursor_new_position.x as usize] = 
                                    !map
                                        [parsed_cursor_new_position.y as usize]
                                        [parsed_cursor_new_position.x as usize];
                            //set map cell to opposite of self
                        }
                        
                    }
                }
                else if let InputResult::Click = input_result {
                    if valid == true && snake_rewrite == false {
                        map
                            [parsed_cursor_new_position.y as usize]
                            [parsed_cursor_new_position.x as usize] = 
                                !map
                                    [parsed_cursor_new_position.y as usize]
                                    [parsed_cursor_new_position.x as usize];
                    } else if cursor.hover == 1 {
                        map = [[false;GAME_FIELD_SIZE];GAME_FIELD_SIZE];
                        reset_map(&mut map);
                        draw_map(&map,GLOBAL_OFFSET_X,GLOBAL_OFFSET_Y);
                        queue!(
                            stdout(),
                            MoveTo(
                                GLOBAL_OFFSET_X + SNAKE_SPAWN_POS_X,
                                GLOBAL_OFFSET_Y + SNAKE_SPAWN_POS_Y,
                            ),
                            Print("   ".on_red())
                        ).handle();
                        stdout().flush().handle();
                    }
                }
                else if let InputResult::Abort = input_result {
                    form = Screen::MainMenu;
                    back_to_main_menu(&buttons_ascii);
                }
                cursor.hover = get_hover(cursor.x,cursor.y,&EDIT_RESET_BUTTON);
                draw(
                    &cursor,
                    if cursor.hover == NONE {
                        COLOR_WHITE
                    } else {
                        COLOR_BLUE
                    }
                );
                stdout().flush().handle();
            }
        }
    }
    // saving map in map.txt
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(FILE_MAP)
        .handle_open(FILE_MAP);
    for line in &map {
        for i in line {
            file.write(if i == &false {b"0"} else {b"1"}).handle_write();
        }
        file.write(b"\n").handle_write();
    }
    file.flush().handle();
    free_window(term_old_w,term_old_h);
}