# hex-encoder-rs
A (WIP) encoder for [Hex Casting](https://modrinth.com/mod/hex-casting) spell serde by macro assistance or hitting some sick ass moves on that DDR pad (ingame).  

Serde algorithms built in collaboration with **@Matt6049**. Huge credits to him, he's basically the brilliant guy behind a lot of this. I would've never conceived it on my own.  

# How to get it
Either:
- download one of the [GitHub releases](https://github.com/Real-Luxof/hex-encoder-rs/tags).
- compile it yourself through [GitHub actions](https://github.com/Real-Luxof/hex-encoder-rs/actions/workflows/build.yml).
- do the following:
    - run `cargo build -r` to build the project in release mode.
    - your output executable is in `target/releases`
    - the only folder that is necessary is `needs` in the project root.
    - put the executable and the `needs` folder in a single folder.
    - done.

# How to use it
## As a CLI tool
Pass in `-h` as an option, and the program will tell you.
## Normally
Click it. Just open the executable.

# How it works
The main idea is that a person:
- gains patterns either through the Hex Casting forums (also known as the [Akashic Records](https://forum.petra-k.at/viewforum.php?f=2)) or through writing a hex and wishes to import the patterns into their Minecraft world,
- writes patterns into a `.hexpattern` file,
- runs this encoder on said file,
- reads the binary output into their world via either:
    - performing "dance moves" (I use the word loosely) and casting their dance Artifact
        - these are sequences of states they put themselves in.  
          (e.g. looking North and moving West while jumping and not sneaking at the same time)
        - each cast the state is taken, translated to binary, and written to their Focus.
        - this has less bitrate than it's alternative and is prone to error.
        - added by me, mostly for whimsy.
    - or running the macro
        - the person fills their hotbar slots 1-8 with Artifacts that write an octal to their Focus.
        - they allow this program to paste in the octals generated for them when they press F6.
        - every 3 ticks (delay due to Artifact cooldown) the script changes slots to write the next octal to their Focus with a right click.
        - this requires no effort.
- runs their decoder hex on their binary,
- then is given the hex and can cast it.
- all without any Hex Casting addons.

By the end, this encoder should be able to output binary, octals, and dance moves, for all valid `.hexpattern` files. This includes `.hexpattern`'s macro functionality.  

Oh, and I apologize for the code. This is like, my first actual Rust project.  
