# Benchmark OpenFST CLI vs RustFST CLI
**Bench parameters** :
- Num warmup rounds : 3
- Num bench runs : 10

**Input FST** : 
- Path : kaldi_models/0001_aspire_chain_model/data/lang_pp_test/G.fst
- Size : 82.53 MB

**Date**: 07/03/19 23:59:39

**Computer specs**:
- Machine type : x86_64
- Platform : Darwin-16.7.0-x86_64-i386-64bit
- Processor : i386
- System : Darwin
## Arcsort
### CLI parameters : ` --sort_type=ilabel`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstarcsort` | 1.383 ± 0.132 | 1.232…1.661 |
| (Rustfst) `rustfst-cli arcsort` | 1.195 ± 0.215 | 0.865…1.494 |
### CLI parameters : ` --sort_type=olabel`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstarcsort` | 1.480 ± 0.158 | 1.297…1.705 |
| (Rustfst) `rustfst-cli arcsort` | 1.164 ± 0.154 | 0.953…1.355 |
## Invert
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstinvert` | 1.769 ± 0.165 | 1.416…1.931 |
| (Rustfst) `rustfst-cli invert` | 1.357 ± 0.140 | 1.129…1.513 |
## Map
### CLI parameters : ` --map_type=arc_sum`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstmap` | 2.406 ± 0.113 | 2.251…2.572 |
| (Rustfst) `rustfst-cli map` | 1.469 ± 0.039 | 1.399…1.523 |
### CLI parameters : ` --map_type=arc_unique`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstmap` | 2.378 ± 0.084 | 2.242…2.492 |
| (Rustfst) `rustfst-cli map` | 1.400 ± 0.063 | 1.285…1.489 |
### CLI parameters : ` --map_type=identity`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstmap` | 2.235 ± 0.110 | 2.128…2.428 |
| (Rustfst) `rustfst-cli map` | 1.386 ± 0.034 | 1.331…1.447 |
### CLI parameters : ` --map_type=input_epsilon`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstmap` | 2.243 ± 0.078 | 2.149…2.372 |
| (Rustfst) `rustfst-cli map` | 1.401 ± 0.132 | 1.160…1.529 |
### CLI parameters : ` --map_type=invert`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstmap` | 2.204 ± 0.065 | 2.073…2.270 |
| (Rustfst) `rustfst-cli map` | 1.336 ± 0.106 | 1.191…1.555 |
### CLI parameters : ` --map_type=output_epsilon`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstmap` | 2.241 ± 0.079 | 2.056…2.329 |
| (Rustfst) `rustfst-cli map` | 1.351 ± 0.022 | 1.321…1.403 |
### CLI parameters : ` --map_type=rmweight`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstmap` | 2.232 ± 0.056 | 2.152…2.323 |
| (Rustfst) `rustfst-cli map` | 1.478 ± 0.095 | 1.336…1.617 |
## Project
### CLI parameters : ` `
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstproject` | 1.917 ± 0.096 | 1.722…2.041 |
| (Rustfst) `rustfst-cli project` | 1.447 ± 0.089 | 1.310…1.548 |
### CLI parameters : ` --project_output=true`
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstproject` | 1.877 ± 0.109 | 1.701…2.053 |
| (Rustfst) `rustfst-cli project` | 1.346 ± 0.087 | 1.246…1.537 |
## Reverse
| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| (Openfst) `fstreverse` | 2.939 ± 0.177 | 2.643…3.166 |
| (Rustfst) `rustfst-cli reverse` | 1.878 ± 0.115 | 1.706…2.013 |
