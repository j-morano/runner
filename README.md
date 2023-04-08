# runner

`runner` is a command line utility that allows you to run any program with single-valued arguments as if they were multi-valued.
More specifically, `runner` takes all the values of the arguments after `--` and combines them, running the program/command with each combination.
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


Additionally, `runner` allows you to filter certain combinations of argument values using the option `--filter-runs`.
For example, the program shown in Snippet 3 would be equivalent to the program shown in Snippet 4.


Snippet 3.
```sh
#!/bin/sh
runner train.py --filter-runs 0.01,16 -- --learning-rate 0.01 0.02 --epochs 4 8 16
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

Finally, you can also tell `runner` to combine only the arguments that are in the same relative position using `--ordered-runner` option.
Note that this requires the lengths of the different lists of argument values to be equal. 
For example, Snippet 5 would be equivalent to Snippet 6.

Snippet 5.
```sh
#!/bin/sh
runner train.py --ordered-runner -- --learning-rate 0.01 0.02 --epochs 4 8
```

Snippet 6.
```sh
#!/bin/sh
# Example program to train a deep learning model with different hyperparameters:
train.py --learning-rate 0.01 --epochs 4
train.py --learning-rate 0.02 --epochs 8
```
