# runner

`runner` is a command line utility that allows you to run any program with single-value arguments as if they were multi-valued. For example, instead of doing this,

```sh
#!/bin/sh
# Example program to train a deep learning model with different hyperparameters:
train.py --learning-rate 0.01 --epochs 4
train.py --learning-rate 0.01 --epochs 8
train.py --learning-rate 0.01 --epochs 16
train.py --learning-rate 0.02 --epochs 4
train.py --learning-rate 0.02 --epochs 8
train.py --learning-rate 0.02 --epochs 16
```

you can do this

```sh
#!/bin/sh
$ runner train.py -- --learning-rate 0.01 0.02 --epochs 4 8 16
```

Additionally, combinations of values can be avoided, or filtered. For example, doing this

```sh
#!/bin/sh
$ runner train.py --filter 0.01,16 @ -- --learning-rate 0.01 0.02 --epochs 4 8 16
```
> NOTE: The at sign `@` is used to indicate the end of the filtering arguments.

is equivalent to do this

```sh
#!/bin/sh
# Example program to train a deep learning model with different hyperparameters:
train.py --learning-rate 0.01 --epochs 4
train.py --learning-rate 0.01 --epochs 8
# No 0.01, 16 combination
train.py --learning-rate 0.02 --epochs 4
train.py --learning-rate 0.02 --epochs 8
train.py --learning-rate 0.02 --epochs 16
```
