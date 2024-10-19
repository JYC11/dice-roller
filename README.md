# TODO in no particular order
1. testing on windows and mac/figure out how to build for windows and mac
2. verbose and shortened output(currently shows verbose by default)
3. some more validation

### Example Commands
* todo: replace debug dice roller with actual program
* inspired by this: https://foundryvtt.com/article/dice-modifiers/
* comparison operators: lt, lte, gt, gte, eq
  * lt = less than
  * lte = less than or equal to
  * gt = greater than
  * gte = greater than or equal to
  * eq = equal to
```shell
target/debug/dice-roller -d 1d20+7 # roll 1d20+7
target/debug/dice-roller -d 1d20+7 -r lt10 # roll 1d20+7 but re-roll if result is less than 10
target/debug/dice-roller -d 1d20+7 -r lt10 --rr # roll 1d20+7 but keep re-rolling if result is less than 10
target/debug/dice-roller -d 10d4 -x lte2  # roll 10d4 and keep re-rolling if result is lte 2
target/debug/dice-roller -d 1d20+7 -x eq15 --xo  # roll 10d4 and re-roll once if result is lte 2
```
* you can also roll multiple dice at once
```shell
target/debug/dice-roller -d 2d6+6d8+9  # roll 2d6+6d8+9
```

```shell
target/debug/dice-roller -d 2d20+5 --kh  # roll 2d20+5, keep highest roll of d20s
target/debug/dice-roller -d 2d20+5 --kl  # roll 2d20+5, keep lowest roll of d20s
target/debug/dice-roller -d 4d6 --dh  # roll 4d6, drop highest roll of d6s
target/debug/dice-roller -d 4d6 --dl  # roll 4d6, drop lowest roll of d6s
target/debug/dice-roller -d 1d20+5 --max 15  # roll 1d20, maximum roll you can get is 15(any roll higher is replaced with 15)
target/debug/dice-roller -d 1d20+5 --min 10  # roll 1d20, minimum roll you can get is 10(any roll lower is replaced with 10)
```
* keep highest, keep lowest, drop highest, drop lowest all default is 1
* you can use it also like this `-d 4d6 --dh 2`, meaning drop highest 2 rolls from 4d6

```shell
target/debug/dice-roller -d 10d20 --cs gt10  # roll 10d20 count successes greater than 10
target/debug/dice-roller -d 10d20 --cf lt10  # roll 10d20 count failures less than 10
target/debug/dice-roller -d 10d20 --even  # roll 10d20 count evens
target/debug/dice-roller -d 10d20 --odd  # roll 10d20 count odds
target/debug/dice-roller -d 10d20 --cf gt10 --df  # roll 10d20 count failures greater than 10, for every failure deduct 1 from final result
target/debug/dice-roller -d 10d20 --sf lt5  # roll 10d20 subtract any rolls from final result that is less than 5
target/debug/dice-roller -d 1d20+15 --ms 10  # roll 1d20+15 with margin of success 10 (eg: 1d20 rolls 14, modifier = 15, 14+15-10 = 19)
```