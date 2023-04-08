# runner

`runner` is a command line utility that allows you to run any program with single-valued arguments as if they were multi-valued.
More specifically, `runner` takes all the values of the arguments and combines them, running the program/command with each combination.
For example, a shell script like the one shown in Snippet 1 could be summarized using `runner` as it is shown in Snippet 2.

Snippet 1.
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

Snippet 2.
```sh
#!/bin/sh
runner train.py -- --learning-rate 0.01 0.02 --epochs 4 8 16
```


Additionally, `runner` allows you to filter certain combinations of argument values.
For example, the program shown in Snippet 3 would be equivalent to the program shown in Snippet 4.


Snippet 3
```sh
#!/bin/sh
runner train.py --filter 0.01,16 @ -- --learning-rate 0.01 0.02 --epochs 4 8 16
# NOTE: The at sign `@` is used to indicate the end of the filtering arguments.
```

Snippet 4.
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
