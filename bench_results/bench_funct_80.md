# Benchmark OpenFST C++ functions vs RustFST Rust functions
**Bench parameters** :
- Num warmup rounds : 3
- Num bench runs : 10

**Input FST** : 
- Path : kaldi_models/0001_aspire_chain_model/data/lang_pp_test/G.fst
- Size : 82.53 MB

**Date**: 07/04/19 00:11:15

**Computer specs**:
- Machine type : x86_64
- Platform : Darwin-16.7.0-x86_64-i386-64bit
- Processor : i386
- System : Darwin
## Arcsort
### CLI parameters : ` --sort_type=ilabel`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.563 ± 0.010 | 0.125 ± 0.006 | 0.760 ± 0.213 | 1.447 ± 0.210 |
| `rustfst` | 0.332 ± 0.018 | 0.035 ± 0.004 | 0.716 ± 0.219 | 1.084 ± 0.229 |
### CLI parameters : ` --sort_type=olabel`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.576 ± 0.033 | 0.134 ± 0.005 | 0.761 ± 0.282 | 1.471 ± 0.274 |
| `rustfst` | 0.336 ± 0.007 | 0.053 ± 0.003 | 0.886 ± 0.125 | 1.276 ± 0.127 |
## Invert
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.591 ± 0.026 | 0.190 ± 0.008 | 1.034 ± 0.218 | 1.815 ± 0.235 |
| `rustfst` | 0.336 ± 0.013 | 0.020 ± 0.001 | 0.757 ± 0.035 | 1.113 ± 0.042 |
## Map
### CLI parameters : ` --map_type=arc_sum`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.574 ± 0.038 | 0.135 ± 0.009 | 0.847 ± 0.045 | 1.556 ± 0.073 |
| `rustfst` | 0.342 ± 0.019 | 0.099 ± 0.008 | 0.774 ± 0.039 | 1.214 ± 0.056 |
### CLI parameters : ` --map_type=arc_unique`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.564 ± 0.039 | 0.130 ± 0.008 | 0.836 ± 0.049 | 1.530 ± 0.062 |
| `rustfst` | 0.340 ± 0.007 | 0.081 ± 0.002 | 0.776 ± 0.014 | 1.197 ± 0.012 |
### CLI parameters : ` --map_type=identity`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.564 ± 0.030 | 0.179 ± 0.006 | 0.873 ± 0.052 | 1.616 ± 0.064 |
| `rustfst` | 0.338 ± 0.014 | 0.053 ± 0.009 | 0.774 ± 0.038 | 1.165 ± 0.046 |
### CLI parameters : ` --map_type=input_epsilon`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.572 ± 0.020 | 0.190 ± 0.011 | 0.841 ± 0.017 | 1.603 ± 0.034 |
| `rustfst` | 0.332 ± 0.014 | 0.072 ± 0.007 | 0.753 ± 0.036 | 1.157 ± 0.049 |
### CLI parameters : ` --map_type=invert`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.554 ± 0.020 | 0.187 ± 0.013 | 0.829 ± 0.038 | 1.569 ± 0.045 |
| `rustfst` | 0.330 ± 0.016 | 0.053 ± 0.004 | 0.777 ± 0.033 | 1.160 ± 0.047 |
### CLI parameters : ` --map_type=output_epsilon`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.573 ± 0.027 | 0.178 ± 0.007 | 0.838 ± 0.026 | 1.589 ± 0.047 |
| `rustfst` | 0.340 ± 0.006 | 0.061 ± 0.014 | 0.782 ± 0.012 | 1.183 ± 0.016 |
### CLI parameters : ` --map_type=rmweight`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.572 ± 0.038 | 0.180 ± 0.008 | 0.842 ± 0.038 | 1.594 ± 0.065 |
| `rustfst` | 0.342 ± 0.006 | 0.063 ± 0.002 | 0.779 ± 0.008 | 1.183 ± 0.011 |
## Project
### CLI parameters : ` `
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.573 ± 0.027 | 0.179 ± 0.007 | 0.850 ± 0.014 | 1.602 ± 0.037 |
| `rustfst` | 0.330 ± 0.015 | 0.027 ± 0.002 | 0.762 ± 0.039 | 1.119 ± 0.047 |
### CLI parameters : ` --project_output=true`
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.580 ± 0.041 | 0.182 ± 0.006 | 0.848 ± 0.028 | 1.610 ± 0.065 |
| `rustfst` | 0.335 ± 0.015 | 0.027 ± 0.001 | 0.766 ± 0.023 | 1.128 ± 0.027 |
## Reverse
| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | 
|:---|---:|---:|---:|---:|
| `openfst` | 0.549 ± 0.013 | 1.297 ± 0.092 | 1.109 ± 0.114 | 2.954 ± 0.159 |
| `rustfst` | 0.329 ± 0.017 | 0.573 ± 0.033 | 0.863 ± 0.094 | 1.765 ± 0.091 |
