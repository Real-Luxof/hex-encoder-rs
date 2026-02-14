# HEADER
Formatted as `[VERSION][CS][OFFSET][SHARED 1s][SHARED 0s][LOCAL MAPPINGS]`
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
After that, there are two bits of instructions.  
### 00 - End (stream/list)
if `STATE.LIST_LEVEL` > 0:  
- `STATE.LIST_LEVEL -= 1`  
- embed everything since the last open left square bracket (kyra reference) as a list

else:  
- end decoding  
### 01 - Embed number
Produced by a Numerical Reflection or embedded number in the original code,  
decoded into an embedded number.  
If born of a Numerical Reflection, encoded into `{ num } flock_disint`.  
If born of an embedded num, encoded into `num`.
there are four bytes after this of the desired number in binary form.
### 10 - Bookkeeper's Gambit
Produced by a Bookkeeper's Gambit in the original code,  
decoded into:
- a swindler's + bookkeeper. (e.g. `5 swindler bk vv`)
- or a `{ desired bookkeeper's gambits }` (e.g. `{ bk v---, bk v-, bk v} run`)  
There are two bytes after this of the `v`s and `-`s represented as 1s and 0s.
### 11 - Embedded list
does the following:
- `STATE.LIST_LEVEL += 1`
- `DECODER_STATE.LISTS_AT.append(pattern_num)`
## EXAMPLE
This:
```k
OP: EMBED LIST
Introspection
    OP: EMBED NUMBER, 10
Retrospection
Explode
OP: END
Jester's Gambit
Thoth's Gambit
```
Becomes:  
```k
{
    { 10 }
    Flock's Disintegration
    Explode
}
Jester's Gambit
Thoth's Gambit
```
