# HEADER
Formatted as `[VERSION][CS][OFFSET][LOCAL MAPPINGS]`
### VERSION
Specifies the version of this encoding.  
### CS (4 bits)
The chunk size. The size of each pattern (before sharing 1s and 0s).  
Want a 5-bit reduced instruction set? Set this to 5.  
Minimum is 5 bits (0000 = 5 bits).  
No defined maximum.  
(Note: the total number of possible patterns is actually CS^2-1 for the null byte.)
### OFFSET (4 bits)
Used with the Local Mappings.  
Root = (offset)(2^CS)
### LOCAL MAPPINGS (algorithm knows)
Local mappings, given by the Matt-cat algorithm.  
Local mappings are a bunch of eight-bit patterns, which are taken by the decoder  
and mapped to `CS`-bit ones.  For example, if `Zone Distillation: Any` is not in the  
current instruction set, it's 8-bit address is added to the local mappings.  
Of course, there is a point beyond which local mappings bloat the hex more than  
stepping up an instruction set would. It is the encoder's task to figure out the  
optimal configuration.  
See `trail1.txt` for a limited example of how to encode and decode in the algorithm.  



# BODY
Mainly composed of PATTERNS. May also contain INSTRUCTIONS.
## PATTERNS
Literally just list patterns lmfao.
## INSTRUCTIONS
All instructions start with a null byte.  
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
### 100-110 - Bookkeeper's Gambit
Instructs the decoder to produce one of these two:
- a swindler's + bookkeeper. (e.g. `5 swindler bk vv`)
- a `desired bookkeeper's gambits` (e.g. `bk v--- bk v- bk v`)  

With `N` bits after where `1 = "v"`, `0 = "-"`, and `N` is:
- when opcode = 100: 4
- when opcode = 101: 8
- when opcode = 110: 16
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
