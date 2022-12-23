# wpkpp

Woodpecker++: alternative VM for woodpecker esolang scripts.

Now with compression, fixed testcases and JSON output.

Original at https://github.com/radical-semiconductor/woodpecker/ and https://radicalsemiconductor.com/woodpecker/

## `.wpkm` syntax

- `INC` is `>`, `INC n` is `n>`
- `CDEC` is `<`, `CDEC n` is `n<`
- `LOAD` is `?` or `v`
- `INV` is `!` or `^`

## Install

```bash
git clone https://github.com/4yn/wpkpp.git
cd wpkpp
cargo install --path .
```

## Usage

**`wpkpp grade [task] [file.(wpk|wpkm)]`**

Grades a woodpecker task. Currently implemented up to stage 5.

Challenge testcases are seeded according to the `WPKPP_SEED` environment variable.

Optional flags:
- `--noprogress`: hide progress bar
- `--nocolor`: disable terminal colors
- `--json`: JSON output

```bash
$ cat 0.wpkm
>?<?>>!
$ wpkpp grade 0 0.wpkm
XOOXXOOOXOOXOOXOXXOXXXOOXOOXXXOXXOXXOOOXOOXOOOOOXOXXXOXXOXOOXOXXXXOOXOXOOOXXOOXOOXOOXXOOOXXXOXXOOXOX
Verdict: WA ❌
Score: 52/100
Instructions: 6
Memory Usage: 3
Instruction counts: INC 2 / CDEC 1 / LOAD 2 / INV 1
Time: Parse 0.000s / VM Setup 0.112s / Grading 4.230s
$ wpkpp grade 0 0.wpkm --json
{"verdict":"WA","score":48,"total":100,"runtime":7,"memory":4,"instructions":{"inc":3,"cdec":1,"load":2,"inv":1},"time_taken":{"parse":0.000043039,"vm":0.118700937,"grade":4.168959827}}
```

**`wpkpp compress [infile.(wpk|wpkm)] [outfile.(wpk|wpkm)]`**

Compresses a woodpecker script to use repeat INC/CDEC instructions.

```bash
$ cat 0.wpkm
>>>>>>>>>?
$ wpkpp compress 0.wpkm 0c.wpkm
Reading file 0.wpkm
Writing to file 0.wpkm
Done
$ cat 0c.wpkm
9>?
```