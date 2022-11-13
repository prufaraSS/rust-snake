# Snake Game written on Rust Language
![cmd_3MLbTtsa0Q](https://user-images.githubusercontent.com/19390500/201515287-c783cc08-304a-444b-a635-ebb35fdc077d.gif) ![image](https://user-images.githubusercontent.com/19390500/201515390-9f68dc6e-6e29-452c-9ee3-cc0527e5f486.png)

## The point of this
The main goal of creating this little game was to learn Rust, but generally speaking, 
I'm writing little games in all programming languages I like, 
because for me it's the best method of learning any language in both entertainment and education fields

## How long did it take
In truth, it took too long, if I knew that before, I probably wouldn't started it at all, but in the end it worth it.  
If I count the days when I've been working all day on this project, I'll assume I've worked for 2-3 months.  
Just before you realize the value of this time, You need to understand that I **wasn't** writing the code this whole time, 
"working on this project" means that I've been learning Rust from zero,
training my skills on little test projects/homeworks that The Rust Programming Language book gave me and mind storming about the project I wanted to make,
so the whole picture could fit in my head.  
I've started it in July 2022 and just got polished it in 13 November 2022 (5 months in total, almost a half of the year!) and during this time,
**1040** lines of code were written.

## The most problematic issues
1. The 'if let' expressions doesn't have reverse variant 'if not let'
2. "Use of moved value" and "the Copy trait is not implemented" errors, which are not always easy to fix

## Game features
### Easy to mod
I've tried to make code as easy to understand as I could, by just changing constant variables you can change almost every game aspect, and you can also skin it by changing files in game folder.
### Ingame map editor
You can change the map layout in the game:
![ezgif com-gif-maker](https://user-images.githubusercontent.com/19390500/201518167-073657bf-bb1d-4c75-a2b0-12248426e513.gif)  
The map will be saved if you close the game with ingame EXIT button (_issue: map won't be saved if closed with window close button [X]_)  
You can also modify map directly in file "map.txt" in game folder (_crash opportunity: map is bigger than it can be_),  
which means you can download custom maps from the internet!

## Unsolvable issues
1. Can't process application closing with window close button which causes changed terminal size and params stay even if you open default cmd.exe
2. Some fonts don't support the most of unicode symbols, so a few elements look messy (_temporary fix: changed apple appearance from 'ó' to '¤'_)

## Credits
Crossterm - Cross-platform Terminal Manipulation Library: https://github.com/crossterm-rs/crossterm#contributing
