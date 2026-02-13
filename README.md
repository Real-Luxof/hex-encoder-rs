A (WIP) encoder for [Hex Casting](https://modrinth.com/mod/hex-casting) spell serde by autohotkey assistance or hitting some sick ass moves on that DDR pad in Minecraft.  

Serde algorithms built in collaboration with **@matt6049**. Huge credits to him, he's basically the brilliant guy behind a lot of this. I would've never conceived it on my own.  

The main idea is that a person:
- gains patterns either through the Hex Casting forums, the [Akashic Records](https://forum.petra-k.at/viewforum.php?f=2) or through writing a hex and wishes to import the patterns into their Minecraft world,
- writes patterns into a `.hexpattern` file,
- runs this encoder on said file,
- reads the binary output into their world via either:
    - performing "dance moves" (I use the word loosely) and casting their dance Artifact
        - these are sequences of states they put themselves in.  
          (e.g. looking North and moving West while jumping and not sneaking at the same time)
        - each cast the state is taken, translated to binary, and written to their Focus.
        - this has less bitrate than it's alternative and is prone to error.
        - added by me, mostly for whimsy.
    - or running an autohotkey script
        - the person fills their hotbar slots 1-8 with Artifacts that write an octal to their Focus.
        - they paste the octals provided by this encoder into the autohotkey script input.
        - every 3 ticks (delay due to Artifact cooldown) the script changes slots to write the next octal to their Focus.
        - this requires no skill.
        - this has insane amounts of bitrate.
- runs their decoder hex on their binary,
- then is given the hex and can cast it.
- all without any addons.

By the end, this encoder should be able to output binary, octals (for the autohotkey script), and dance moves for easy-medium-hard dance decoders, for all valid `.hexpattern` files. This includes macro functionality.
