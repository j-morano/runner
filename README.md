# runner

`runner` is a command line utility that allows you to run any program with single-value arguments as if they were multi-valued. For example:
```sh
# Example program to train a deep learning model with only one option per
#  argument:
$ train.py --learning-rate 0.01 --num-epochs 4 --model resnet
# Now, using runner:
$ runner train.py --learning-rate 0.01 0.02 --num-epochs 4 8 --model resnet vgg
```
