# csv-transformer

Command line tool to rearrange CSV files in certain ways.

This was developed to help transforming
Simplified Chinese survey result of [2020 State of Rust survey][survey] to match the global version.

[survey]: https://blog.rust-lang.org/2020/09/10/survey-launch.html

## Usage

Firstly, with a CSV file, use the following command to extract all the columns:
```bash
csv-transformer extract original.csv > transform.yaml
```

This will transform a CSV file like
```csv
Question 1,Question 2,Question 3
Answer B,Answer X,
Answer A,Answer X,Random text
Answer A,Answer Y,
```
into
```yaml
- "A: Question 1"
- "B: Question 2"
- "C: Question 3"
```

The strings formatted `X: Header text` is called a column reference,
and the letters before the first colon is the column index.

You can then edit the YAML file to reflect the transformation you want.
Please refer to the [transformations][#Transformations] section for available transformations.
This is an example of transformation file we used for the survey:
[transform.yaml](https://gist.github.com/upsuper/3e90f78d84b84c9741d585a1d462b1b5).

After you edit it, you can use the following command to generate the result:
```bash
csv-transformer transform original.csv transform.yaml > result.csv
```

### Transformations

Each item in the YAML file represents a rule
to generate one or more columns in the transformation result in its order.
If it's kept untouched (just a column reference),
the column would be preserved as is.
Otherwise, it can be one of the following transformations.

#### Rename

A rename transformation basically just changes the header text.

Example:
```yaml
- transform: rename
  header: "New Header"
  column: "A: Old Header"
```
transforms

| Old Header |
| ---------- |
| Value 1    |
| Value 2    |

to

| New Header |
| ---------- |
| Value 1    |
| Value 2    |

#### Timestamp

A timestamp transformation reformats the date and time value
with the format of [`strftime`](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html).

Example:
```yaml
- transform: timestamp
  column: "A: Timestamp"
  from: "%d-%b-%Y %H:%M:%S"
  to: "%d/%m/%Y %H:%M:%S"
```
transforms

| Timestamp            |
| -------------------- |
| 26-Sep-2020 01:12:42 |
| 25-Sep-2020 23:23:52 |

to

| Timestamp           |
| ------------------- |
| 26/09/2020 01:12:42 |
| 25/09/2020 23:23:52 |

Optionally, you can also provide a `header` field to rename the column at the same time.

#### Join

A join transformation concatenates values from multiple columns into a single column.

Example:
```yaml
- transform: join
  header: "Question?"
  columns:
  - "A: Question? Rust"
  - "B: Question? C++"
  - "C: Question? Python"
```
transforms

| Question? Rust | Question? C++ | Question? Python |
| -------------- | ------------- | ---------------- |
| Rust           |               | Python           |
|                | C++           |                  |
| Rust           | C++           | Python           |

to

| Question?         |
| ----------------- |
| Rust, Python      |
| C++               |
| Rust, C++, Python |

Optionally, you can provide a `sep` field to change the default separator `, ` to something else.

It's also possible to slightly format the values from columns before joining
via replacing the column reference item with a object.

Example:
```yaml
- transform: join
  header: "Conference anywhere?"
  columns:
  - column: "A: Conference in China?"
    format: "China - {}"
  - column: "B: Conference outside China?"
    format: "outside China - {}"
```
transforms

| Conference in China? | Conference outside China? |
| -------------------- | ------------------------- |
| Yes                  | No                        |
| Maybe                |                           |
|                      | No                        |

to

| Conference anywhere?            |
| ------------------------------- |
| China - Yes, outside China - No |
| China - Maybe                   |
| outside China - No              |

#### Transpose

A transpose transformation transposes values and their header across several columns.

Example:
```yaml
- transform: transpose
  sources:
    "A: Question? 1st": 1st
    "B: Question? 2nd": 2nd
    "C: Question? 3rd": 3rd
  columns:
    "Question? Go": Go
    "Question? C++": C++
    "Question? Rust": Rust
    "Question? Python": Python
```
transforms

| Question? 1st | Question? 2nd | Question? 3rd |
| ------------- | ------------- | ------------- |
| Rust          | C++           | Go            |
| Python        | Go            | Rust          |

to

| Question? Go | Question? C++ | Question? Rust | Question? Python |
| ------------ | ------------- | -------------- | ---------------- |
| 3rd          | 2nd           | 1st            |                  |
| 2nd          |               | 3rd            | 1st              |

If a value is present in multiple source columns,
the first matching one would be picked.

An error would be raised if a non-empty value in the source columns can't be mapped to a target column.

## License

Copyright (C) 2020 Xidorn Quan

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
