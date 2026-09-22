# CS4414 HW 2 - Fast edit distances

## Logistics

- This homework may be done individually or with a partner (recommended)
- Task 1 will be due Mon, Sep 21 at 11:59 PM via Gradescope
- The full submission is due Mon, Sep 28 at 11:5p PM via Gradescope

## Changelog

2026-09-17:
- Fix typo and broken link in step 3

2026-09-16:
- Initial project release

## Introduction

An *edit distance* between two strings is a distance measure based on
how many editing steps are needed to convert one string into another.
This is a true metric: it is symmetric, the distance between a string
and itself is zero, the distance between different strings is nonzero,
and it satisfies the triangle inequality.  There are many possible
such distance measures; we consider the [Hamming distance][hamming],
which is just the number of character substitutions needed to change
one word into another.  For example, `tree` and `true` are distance 1
from each other, because a single character substitution (`e` to `u`)
changes one to the other.  We can define Hamming distance for strings
of different lengths as well; the distance between `true` and `truth`
is 2; we change the `e` to a `t` and a blank character to an `h`.

Hamming distance is a simple case of a high-dimensional pairwise
distance calculations.  These types of calculations are ubiquitous in
machine learning applications, where we often use distances
between feature vectors (or something like a cosine between feature
vectors, depending on the use case) as a way to judge similarities
between pairs of objects.

For this assignment, we will compute a measure of "centrality" for
strings in a dictionary (we will use the same dictionaries of
normalized English words that we saw in the previous assignments).
The centrality measure we use will be the mean distance between each
word and every other word in the dictionary.

## Setup

Directions for the assignment are in this file, and starter code is in
`src/hamming.rs`.  We provide both a `main.rs` file for a command-line
driver and a `lib.rs` file for automated testing.  Code submitted to
Gradescope will undergo basic correctness testing, but timing will be
done by a separate process on Google Compute Engine nodes.

There are questions for you in the prompt.  Please fill out your
answers in the `WRITEUP.md` file.

For testing, you may run the driver on the `popular.txt` dictionary
and compare the output to `tests/reference.txt`.  You should also
write unit tests in the `hamming.rs` module.

Timing runs should be done on a `c4d-highcpu-2` node on Google Compute
Engine.  We will be using a default Debian build as our virtual
machine OS for the timing runs, and the most recent Rust version at
the time of this posting (1.98.1).

Scoring will involve a writeup and a performance component.
There are eight questions in the `WRITEUP.md`; that file should be
completed with your answers.  There is also one point for submitting
a crate that compiles (the autograder will do some basic correctness
checks, but not for a score).

Performance scoring thresholds (C4D and the `enable1` dictionary)

| Score (of 2) | Blocked | Packed | SWAR |
|--------------|---------|--------|------|
| 3            | 175s    | 140s   | 70s  |
| 2            | 185s    | 150s   | 80s  |
| 1            | 195s    | 160s   | 90s  |

No credit will be given for code that computes the wrong answers.

## Steps

Steps 0-1 should be completed by 9/21 at 11:59 PM.  The remaining
steps should be computed for a final submission by 9/28 at 11:59 PM.

### Step 0: Build modes

We have provided an untuned starter code as a starting point.

1. For the dictionary `examples/popular.txt`, how long does it take to
   run the code in the `debug` compilation mode?  What about the
   `release` mode?

2. Based on the relative sizes of the dictionaries, estimate how long
   you think it would take to run in the two modes for the
   `examples/enable1.txt` dictionary.  Test your hypothesis in release
   mode; how long does it actually take to run?

### Step 1: Blocking

Part (not all) of the speed of the untuned code involves the cache
utilization.  The full dictionary is too big to fit into L1 cache;
as we will see in lecture, *tiling* (or *blocking*) the computation
lets us reduce the cache miss rate.

1. What is the estimated memory footprint for the two dictionaries
   (`popular.txt` and `enable1.txt`)?  Include the storage for the
   `String` metadata in your accounting (don't worry about storage for
   the allocator's data structures).

2. Complete the code for the blocked variant.  You should see a speed
   difference that is noticeable, but not enormous.  What difference
   do you see?

### Step 2: Removing indirection

The naive code suffers a performance penalty due to pointer chasing:
accessing a string involves looking up the string metadata in an
array, and then going to look up the string in memory.  Moreover,
there is a penalty for dealing with variable length strings and
Unicode encodings.  None of these penalties are that large, but they
are all in an inner loop!

Complete the function `XXX` to convert the `Vec<String>` dictionary
into a dictionary where the data is stored inline (vs storing a
pointer to a heap-allocated string).  You may assume that the words in
the `Vec<String>` have already been normalized such that they consist
of ASCII-encoded lower-case letters.  You should make sure that you
can accommodate the longest strings in the `enable1.txt` dictionary,
but you do not necessarily need to accommodate anything longer.  You
should also complete the `XXX` function so that it computes the
Hamming distance based on your new coding.

1. What is the speed difference compared to the method in step 2?

2. The longest word in `enable1.txt` is 28 characters, but most are
   shorter.  If you write your code to reserve one byte for the word
   length at the beginning, what type of performance improvement do
   you see?

### Step 3: Packed representations

At least in principle, we can use ["bit-twiddling hacks"][hacks]
for [SIMD within a register (SWAR)][swar] computations of things
like letter-by-letter Hamming distances.  One such scheme goes as
follows:

- Represent each letter by an index `1..=26`, with `0` reserved for a
  null.  This can be represented within five bits, but we reserve a
  sixth bit to assist in our counting scheme.  In this way, we
  can represent five characters in a single 32-bit integer or ten in a
  single 64-bit integer.
- We can compare two such encodings by XORing the bit patterns.
  If the XOR is zero in a given six-bit field, the characters for that
  field were the same; otherwise they are different.  For each such field
  we add `0b011111`; if the add generates a carry into the highest
  order bit, then the XOR result in the field was nonzero.
- Mask with `0b100000`, shift the entire integer right by six bits,
  and mod by 63 to get the final count.

Use this as a building block to complete the `dist` routine for
comparing two words via packed representations.  We provide the code
to pack a string into the desired encoding, and leave to you the task
of doing the distance computation.

People often resort to bit-twiddling hacks because they think it will
improve performance.  Sometimes this works; sometimes the intuition is
misguided.  What do you see?  Is your version any faster than the
method you explored in Step 2?

[hacks]: https://graphics.stanford.edu/~seander/bithacks.html
[swar]: https://en.wikipedia.org/wiki/SWAR

### Step 4: Speed demon!

For the final step, see how fast you can get this to go!  This may
involve reducing the total number of computations (e.g. the distance
matrix is symmetric, and the starter code does not take advantage of
that); making each individual distance computation faster; or making
better use of the memory subsystem.  Keep a record of what you tried
and your explanation of why it worked (or did not).

The fastest code gets bragging rights!
