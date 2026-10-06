# beet_eval

Evaluates a subject against packages of evals and rubrics: checks, grades, results and the next unit of work, with markdown documents as the first subject and Office forms as the renderer

A downstream of [beet](https://github.com/mrchantey/beet), depended on by path: this repo's own beet binary (`just cli`) is the stock runner plus `EvalPlugin`, serving `main.bsx`.

```sh
just cli --help
just test
```
