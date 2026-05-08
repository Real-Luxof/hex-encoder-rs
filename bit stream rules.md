# HEADER
Formatted as `[VERSION][CS][OFFSET][LOCAL MAPPINGS]`
### VERSION (8 bits)
Specifies the version of this encoding. This version of the bit stream rules is written for `0`.  
### CS (4 bits)
The chunk size. The size of each pattern (before sharing 1s and 0s).  
Want a 5-bit reduced instruction set? Set this to 5.  
No defined maximum.  
(Note: the total number of possible patterns is actually CS^2-1 for the null chunk.)
### LOCAL MAPPINGS (algorithm-defined)
Local mappings are a bunch of 8-bit patterns sorted in ascending order and compressed into a  
binary tree. These are taken by the decoder and mapped to the end `CS`-bit ones. For example,  
if `Zone Distillation: Any` isn't in the current instruction set but present in the hex being  
serialized, its 8-bit address is added to the local mappings which the decoder takes and maps  
to the end of its `CS`-bit instruction set, basically putting `Zone Distillation: Any` in our  
`CS`-bit reduced instruction set.  

The process of compression into the binary tree and the little bit of compression done within  
is handled by the Matt-cat algorithm. Of course there's a point at which local mappings bloat  
the binary and it would be better to step up an instruction set instead. It's the task of the  
encoder to figure out the optimal configuration for smallest binary.  



# BODY
Mainly composed of PATTERNS. May also contain INSTRUCTIONS.
## PATTERNS
Literally just list patterns lmfao.
## INSTRUCTIONS
All instructions start with a null chunk.  
After that, there are three bits that say what instruction is being given.  
### 000 - End (stream/list)
if `STATE.LIST_LEVEL` > 0:  
- `STATE.LIST_LEVEL -= 1`  
- embed everything since the last open left square bracket (kyra reference) as a list

else:  
- end decoding  
### 001-011 - Embed Number (varying sizes)
Instructs the decoder to produce a number iota and add it to the list.  
Handling of Numerical Reflection via `{n} flock_disint` is done by the encoder.  
- 001: 8-bit integer.
- 010: 32-bit float.
- 011: 64-bit double.
### 100-101 - Bookkeeper's Gambit
Instructs the decoder to produce one of these two:
- a swindler's + bookkeeper. (e.g. `5 swindler bk vv`)
- a `desired bookkeeper's gambits` (e.g. `bk v--- bk v- bk v`)  

With `N` bits after where `1 = "v"`, `0 = "-"`, and `N` is:
- when opcode = 100: 4
- when opcode = 101: 16
### 110 - Vector
Instructs the decoder to produce a vector iota and add it to the list.  
Following this instruction are always three pseudo-Embed Number instructions to represent
the components of the vector.
- 00: 8-bit integer.
- 01: 32-bit float.
- 10: 64-bit double.
### 111 - Start list
does the following:
- `STATE.LIST_LEVEL += 1`
- `DECODER_STATE.LISTS_AT.append(pattern_num)`
## EXAMPLE
This:
```k
Mind's Reflection
Compass' Purification
Consideration
OP: EMBED LIST
    Introspection
        OP: EMBED NUMBER, 10
    Retrospection
    Flock's Disintegration
    Explode
OP: END
Hermes' Gambit
OP: END
```
Becomes:  
```k
Mind's Reflection
Compass' Purification
Consideration
[
    {
        10
    }
    Flock's Disintegration
    Explode
]
Hermes' Gambit
```
