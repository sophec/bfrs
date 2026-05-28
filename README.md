# Shitty Brainfuck to C Transpiler

Does some simple "optimizations" (as if your compiler couldn't do them
itself... perhaps "simplifications" would be more accurate):

- Multiple additions/subtractions are combined into one line
- Multiple pointer moves are combined into one line
- Move(s) followed by addition(s) or subtraction(s) combined into one line
- Loops immediately at the start of the program are removed

## Example

Given `helloworld.bf`:

```brainfuck
[ This program prints "Hello World!" and a newline to the screen; its
  length is 106 active command characters. [It is not the shortest.]

  This loop is an "initial comment loop", a simple way of adding a comment
  to a BF program such that you don't have to worry about any command
  characters. Any ".", ",", "+", "-", "<" and ">" characters are simply
  ignored, the "[" and "]" characters just have to be balanced. This
  loop and the commands it contains are ignored because the current cell
  defaults to a value of 0; the 0 value causes this loop to be skipped.
]
++++++++                Set Cell #0 to 8
[
    >++++               Add 4 to Cell #1; this will always set Cell #1 to 4
    [                   as the cell will be cleared by the loop
        >++             Add 2 to Cell #2
        >+++            Add 3 to Cell #3
        >+++            Add 3 to Cell #4
        >+              Add 1 to Cell #5
        <<<<-           Decrement the loop counter in Cell #1
    ]                   Loop until Cell #1 is zero; number of iterations is 4
    >+                  Add 1 to Cell #2
    >+                  Add 1 to Cell #3
    >-                  Subtract 1 from Cell #4
    >>+                 Add 1 to Cell #6
    [<]                 Move back to the first zero cell you find; this will
                        be Cell #1 which was cleared by the previous loop
    <-                  Decrement the loop Counter in Cell #0
]                       Loop until Cell #0 is zero; number of iterations is 8

The result of this is:
Cell no :   0   1   2   3   4   5   6
Contents:   0   0  72 104  88  32   8
Pointer :   ^

>>.                     Cell #2 has value 72 which is 'H'
>---.                   Subtract 3 from Cell #3 to get 101 which is 'e'
+++++++..+++.           Likewise for 'llo' from Cell #3
>>.                     Cell #5 is 32 for the space
<-.                     Subtract 1 from Cell #4 for 87 to give a 'W'
<.                      Cell #3 was set to 'o' from the end of 'Hello'
+++.------.--------.    Cell #3 for 'rl' and 'd'
>>+.                    Add 1 to Cell #5 gives us an exclamation point
>++.                    And finally a newline from Cell #6
```

Turn this into C:

```sh
cargo run -- helloworld.bf >helloworld.c
```

Or make it readable:

```sh
cargo run -- helloworld.bf | clang-format >helloworld.c
```

Or compile it right away:

```sh
cargo run -- helloworld.bf | gcc -x c -o helloworld -
```

The output (piped through clang-format):

```c
#include <stdio.h>
static char a[65535] = {0};
static char* p = a;
int main() {
  *p += 8;
  for (; *p;) {
    *++p += 4;
    for (; *p;) {
      *++p += 2;
      *++p += 3;
      *++p += 3;
      ++*++p;
      --*(p -= 4);
    }
    ++*++p;
    ++*++p;
    --*++p;
    ++*(p += 2);
    for (; *p;) {
      --p;
    }
    --*--p;
  }
  p += 2;
  putchar(*p);
  *++p -= 3;
  putchar(*p);
  *p += 7;
  putchar(*p);
  putchar(*p);
  *p += 3;
  putchar(*p);
  p += 2;
  putchar(*p);
  --*--p;
  putchar(*p);
  --p;
  putchar(*p);
  *p += 3;
  putchar(*p);
  *p -= 6;
  putchar(*p);
  *p -= 8;
  putchar(*p);
  ++*(p += 2);
  putchar(*p);
  *++p += 2;
  putchar(*p);
}
```
