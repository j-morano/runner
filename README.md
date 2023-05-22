# runner

`runner` is a command line utility that allows you to run any program with single-valued arguments as if they were multi-valued.
More specifically, `runner` takes all the values of the arguments after `--` and combines them, running the program/command with each combination.
For example:

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

## Multiple commands and command-specific options

You can introduce multiple commands separated by ` , ` (space-comma-space) with shared arguments and introduce command-specific options.
An option is considered command-specific if there is a number and a comma after the dash (e.g. `--1,option`, `-1,o`).
The number before the comma indicates the index (starting by 0) of the command with which it is associated.
For example:

```sh
runner train.py , eval.py -- --model vgg resnet --learning-rate 0.1,0.2,0.3 --epochs 8,12,16 --1,test-data ImageNet CIFAR-10

# is equivalent to

train.py --model vgg --learning-rate 0.1 --epochs 8
train.py --model vgg --learning-rate 0.2 --epochs 12
train.py --model vgg --learning-rate 0.3 --epochs 16
train.py --model resnet --learning-rate 0.1 --epochs 8
train.py --model resnet --learning-rate 0.2 --epochs 12
train.py --model resnet --learning-rate 0.3 --epochs 16
eval.py --model vgg --learning-rate 0.1 --epochs 8 --test-data ImageNet
eval.py --model vgg --learning-rate 0.2 --epochs 12 --test-data ImageNet
eval.py --model vgg --learning-rate 0.3 --epochs 16 --test-data ImageNet
eval.py --model resnet --learning-rate 0.1 --epochs 8 --test-data ImageNet
eval.py --model resnet --learning-rate 0.2 --epochs 12 --test-data ImageNet
eval.py --model resnet --learning-rate 0.3 --epochs 16 --test-data ImageNet
eval.py --model vgg --learning-rate 0.1 --epochs 8 --test-data CIFAR-10
eval.py --model vgg --learning-rate 0.2 --epochs 12 --test-data CIFAR-10
eval.py --model vgg --learning-rate 0.3 --epochs 16 --test-data CIFAR-10
eval.py --model resnet --learning-rate 0.1 --epochs 8 --test-data CIFAR-10
eval.py --model resnet --learning-rate 0.2 --epochs 12 --test-data CIFAR-10
eval.py --model resnet --learning-rate 0.3 --epochs 16 --test-data CIFAR-10
# Notice that '--test-data' option is only combined with 'eval.py' command.
```

## Distributed arguments

You can combine multiple arguments of an option in a single command depending on the number of runners for parallel processing. That is, the list of arguments for the option is split in approximately equal parts that are then distributed among the number of runners. To use this feature, you have to put the character `%` after the dashes of the option (e.g., `-%o`, `--%option`). For example:


```sh
runner --runners 1 preprocess.py -- --%data-dirs ImageNet CIFAR-10 Places Oxford102Flower CelebA Caltech-256

# is equivalent to

preprocess.py --data-dirs ImageNet CIFAR-10 Places Oxford102Flower CelebA Caltech-256

#--------------------------------------

runner --runners 2 preprocess.py -- --%data-dirs ImageNet CIFAR-10 Places Oxford102Flower CelebA Caltech-256

# is equivalent to

preprocess.py --data-dirs ImageNet Places CelebA
preprocess.py --data-dirs CIFAR-10 Oxford102Flower Caltech-256

# Notice that both commands are run in parallel.

#--------------------------------------

runner --runners 3 preprocess.py -- --%data-dirs ImageNet CIFAR-10 Places Oxford102Flower CelebA Caltech-256

# is equivalent to

preprocess.py --data-dirs ImageNet Oxford102Flower
preprocess.py --data-dirs CIFAR-10 CelebA
preprocess.py --data-dirs Places Caltech-256

# Again, these commands are run in parallel.
```


## Other options:

- `--dry-runner`: Print the commands that would be executed without actually executing them.
- `--runners`: Number of commands to run in parallel.
- `--bg-runner`: Run the commands in the background.
