# CS4414 HW 2 - Fast edit distances

You should complete the following questions in your final submission:

## Step 0: Build modes

1. For the dictionary `examples/popular.txt`, how long does it take to
   run the code in the `debug` compilation mode?  What about the
   `release` mode?

   | Build mode | Computation time |
   | --- | ---: |
   | Debug | 96.616 s |
   | Release | 4.196 s |

2. Based on the relative sizes of the dictionaries, estimate how long
   you think it would take to run in the two modes for the
   `examples/enable1.txt` dictionary.  Test your hypothesis in release
   mode; how long does it actually take to run?

   `popular.txt` contains 25,322 words and `enable1.txt` contains 172,823
   words. Since every word is compared with every word, I estimated the
   runtime multiplier as `(172823 / 25322)^2 = 46.581`.

   | Build mode | Predicted enable1 time |
   | --- | ---: |
   | Debug | 4500.448 s (about 75.01 minutes) |
   | Release | 195.447 s (about 3.26 minutes) |

   The actual release computation took **212.077 seconds** (about
   **3 minutes 32 seconds**), approximately **8.5% longer** than predicted.

## Step 1: Blocking

1. What is the estimated memory footprint for the two dictionaries
   (`popular.txt` and `enable1.txt`)?  Include the storage for the
   `String` metadata in your accounting (don't worry about storage for
   the allocator's data structures).

   On the 64-bit VM, each Rust `String` has **24 bytes of metadata**:
   a pointer, length, and capacity. Each dictionary character uses one
   byte because the words contain only ASCII letters. The outer `Vec`
   adds another 24 bytes.

   ```text
   Estimated memory = total letter bytes + 24 × number of words + 24
   ```

   | Dictionary | Words | Letter bytes | Estimated memory |
   | --- | ---: | ---: | ---: |
   | `popular.txt` | 25,322 | 185,196 | 792,948 bytes (0.756 MiB) |
   | `enable1.txt` | 172,823 | 1,570,540 | 5,718,316 bytes (5.453 MiB) |

   These estimates exclude unused allocation capacity and allocator
   bookkeeping. Both dictionaries exceed the VM's 48 KiB L1 data cache,
   which motivates processing smaller groups of words.

2. Complete the code for the blocked variant.  You should see a speed
   difference that is noticeable, but not enormous.  What difference
   do you see?

   I implemented blocking using the starter's **500-word block size**.
   The program processes each pair of blocks before moving on, allowing
   it to reuse dictionary data in cache. It still compares every word
   with every word using the original distance function.

   Using release mode on the same `c4d-highcpu-2` VM:

   | Dictionary | Original | Blocked | Speedup |
   | --- | ---: | ---: | ---: |
   | `popular.txt` | 4.196 s | 2.406 s | 1.74× |
   | `enable1.txt` | 212.077 s | 138.920 s | 1.53× |

   Blocking reduced computation time by **42.7%** and **34.5%**,
   respectively. This is a noticeable improvement without reducing
   the number of comparisons.

## Step 2: Removing indirection

1. What is the speed difference compared to the method in step 2?

2. The longest word in `enable1.txt` is 28 characters, but most are
   shorter.  If you write your code to reserve one byte for the word
   length at the beginning, what type of performance improvement do
   you see?

## Step 3: Packed representation

What do you see?  Is your version any faster than the
method you explored in Step 2?

## Step 4: Speed demon

Describe the steps that you took to get to your final optimized
version!
