dawg
====

A rust implementation of DAWG represented by double-array.

DAWG is the abbreviation of "Directed Acyclic Word Graph" and
double-array is an efficient representation of tries.

See also [cl-dawg](https://github.com/sile/cl-dawg) which is the original common-lisp version.

Build
-----

You can build the project with [cargo](https://github.com/rust-lang/cargo) command.

```sh
$ git clone git@github.com:sile/rust-dawg.git
$ cd rust-dawg
$ cargo build --release
$ ls target/release/dawg_*
target/release/dawg_build  target/release/dawg_search
```

Usage Examples
--------------

### Build DAWG index file

```sh
# Gets n-gram corpus
$ wget -xnH -i http://dist.s-yata.jp/corpus/nwc2010/ngrams/char/over99/filelist
$ xz -d corpus/nwc2010/ngrams/char/over999/*/*.xz
$ head -9 corpus/nwc2010/ngrams/char/over999/2gms/2gm-0000
" "     123067
" #     2867
" $     1047
" %     2055
" &     3128
" '     3730
" (     134099
" )     40850
" *     5752

$ cut -f 1 corpus/nwc2010/ngrams/char/over999/*gms/* | sed -e 's/ //g' > words.tmp
$ LC_ALL=c sort words.tmp -o words

$ wc -l words
44280717 words
$ du -h words
652M    words

# Builds index file
$ /usr/bin/time -v target/release/dawg_build dawg.idx < words
DONE
        Command being timed: "target/release/dawg_build /tmp/dawg.idx"
        User time (seconds): 61.36
        System time (seconds): 0.56
        Percent of CPU this job got: 100%
        Elapsed (wall clock) time (h:mm:ss or m:ss): 1:01.88
        Average shared text size (kbytes): 0
        Average unshared data size (kbytes): 0
        Average stack size (kbytes): 0
        Average total size (kbytes): 0
        Maximum resident set size (kbytes): 1921308
        Average resident set size (kbytes): 0
        Major (requiring I/O) page faults: 4
        Minor (reclaiming a frame) page faults: 3199
        Voluntary context switches: 10
        Involuntary context switches: 81
        Swaps: 0
        File system inputs: 568
        File system outputs: 409824
        Socket messages sent: 0
        Socket messages received: 0
        Signals delivered: 0
        Page size (bytes): 4096
        Exit status: 0
$ du -h dawg.idx
201M    dawg.idx
```

### Execute common-prefix search

```sh
$ target/release/dawg_search dawg.idx
> unknown
[3569979] u
[3571601] un
[3571823] unk
[3571830] unkn
[3571831] unkno
[3571832] unknow
[3571833] unknown

> プログラミング言語
[25452934] プ
[25516667] プロ
[25518828] プログ
[25518857] プログラ
[25518935] プログラミ
[25518936] プログラミン
[25518937] プログラミング

> Ctrl+D  # quit
```

TODO
----

- Optimize query functions
