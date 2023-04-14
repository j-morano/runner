# runner

`runner` is a command line utility that allows you to run any program with single-valued arguments as if they were multi-valued.
More specifically, `runner` takes all the values of the arguments after `--` and combines them, running the program/command with each combination.
For example:

Snippet 1.
```sh
runner train.py -- --learning-rate 0.01 0.02 --epochs 4 8 16

# is equivalent to

train.py --learning-rate 0.01 --epochs 4
train.py --learning-rate 0.01 --epochs 8
train.py --learning-rate 0.01 --epochs 16
train.py --learning-rate 0.02 --epochs 4
train.py --learning-rate 0.02 --epochs 8
train.py --learning-rate 0.02 --epochs 16
```

## --filter-runs

Additionally, `runner` allows you to filter certain combinations of argument values using the option `--filter-runs`.
For example:

```sh
runner train.py --filter-runs 0.01,8 0.01,16 -- --learning-rate 0.01 0.02 --epochs 4 8 16

# is equivalent to

train.py --learning-rate 0.01 --epochs 4
# No 0.01,8 combination
# No 0.01,16 combination
train.py --learning-rate 0.02 --epochs 4
train.py --learning-rate 0.02 --epochs 8
train.py --learning-rate 0.02 --epochs 16
```

You can combine multiple filtering values with `+`, so that the previous command could be also written as follows:
```sh
runner train.py --filter-runs 0.01,8+16 -- --learning-rate 0.01 0.02 --epochs 4 8 16
```

## --allow-runs

Instead of filtering runs, you can tell `runner` that some option values can be combined only with other specific option values.
The first element in the 'allow' rule is the one that has to be combined only with the other option values that follow.
Thus, which value goes first is very important.
Some examples:

```sh
train.py --allow-runs vgg,0.1 -- --model vgg resnet --learning-rate 0.1,0.2 --epochs 4 8

# is equivalent to

train.py --model vgg --epochs 4 --learning-rate 0.1
train.py --model vgg --epochs 8 --learning-rate 0.1
train.py --model resnet --epochs 4 --learning-rate 0.1
train.py --model resnet --epochs 4 --learning-rate 0.2
train.py --model resnet --epochs 8 --learning-rate 0.1
train.py --model resnet --epochs 8 --learning-rate 0.2
# Notice that vgg model is only combined with learning rate 0.1.
```
Notice the difference between this command and the following one:

```sh
train.py --allow-runs 0.1,vgg -- --model vgg resnet --learning-rate 0.1,0.2 --epochs 4 8

# is equivalent to

train.py --model vgg --epochs 4 --learning-rate 0.1
train.py --model vgg --epochs 4 --learning-rate 0.2
train.py --model vgg --epochs 8 --learning-rate 0.1
train.py --model vgg --epochs 8 --learning-rate 0.2
train.py --model resnet --epochs 4 --learning-rate 0.2
train.py --model resnet --epochs 8 --learning-rate 0.2
# Notice that, in this, the learning rate 0.1 is only combined with vgg model.
```


## --ordered-runner

You can also tell `runner` to combine only the arguments that are in the same relative position using `--ordered-runner` option.
Note that this requires the lengths of the different lists of argument values to be equal. 
For example:

```sh
runner train.py --ordered-runner -- --learning-rate 0.01 0.02 --epochs 4 8

# is equivalent to

train.py --learning-rate 0.01 --epochs 4
train.py --learning-rate 0.02 --epochs 8
```

## Positional arguments

You can also set positional option values.
Positional values are combined only with the positional values in the same relative position.
For example:

```sh
runner train.py -- --model vgg resnet --learning-rate 0.1,0.2,0.3 --epochs 8,12,16

# is equivalent to

train.py --model vgg --learning-rate 0.1 --epochs 8
train.py --model vgg --learning-rate 0.2 --epochs 12
train.py --model vgg --learning-rate 0.3 --epochs 16
train.py --model resnet --learning-rate 0.1 --epochs 8
train.py --model resnet --learning-rate 0.2 --epochs 12
train.py --model resnet --learning-rate 0.3 --epochs 16
# Notice that 0.1 is always combined with 8, 0.2 with 12 and 0.3 with 16.
```
