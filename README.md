# aparsh

#### parsing bash arguments without the hassle
##### synopsis
create a `toml` file, fill in the data and youre done!


**example file with all the options**
```toml
name = "foo" # required
description = "the best foo of all the naming masterpieces"
prologue = "thank you for coming to my ted talk"

# there's no requirement for having any positional args nor flags
[[positional]]
name = "bar" # required
description = "for setting the _bar_ lower"
varname = "definitely_not_bar"

# example for a boolean flag
[[flag]]
name = "list" # required
short = "l"
description = "list all possible variables in existence"
varname = "LIST_FLAG"
type = "store_true"

# example for multiple options
[[flag]]
name = "drink" # required
description = "I am your bartender, what would you like to drink?"
choice = ["water", "beer", "wine", "coke"] # there's only pepsi
default = "beer"

```

Running
```sh
$ ./aparsh foo.toml > foo.sh
```

Would yield:
```sh

#!/bin/sh

__usage="foo - the best foo of all the naming masterpieces
Usage: foo BAR [OPTIONS]

\e[4mArguments:\e[0m
  BAR                      for setting the _bar_ lower

\e[4mOptions:\e[0m
  -l, --list               list all possible variables in existence
  --drink DRINK            I am your bartender, what would you like to drink?
  --help                   show this message and exit.

thank you for coming to my ted talk"

HAS_HELP_FLAG=$(echo "$@" | grep -c "\--help")
if [[ $HAS_HELP_FLAG -gt 0 ]]; then
  echo -e "$__usage"
      exit
fi

if [ $# -lt 1 ]; then
  echo "Not enough arguments - expected 1, recieved $#"
  exit 1
fi

# initialize positional arguments
DEFINITELY_NOT_BAR="$1"
if [[ "$DEFINITELY_NOT_BAR" =~ ^(-|--) ]]; then
  echo "Not enough arguments - expected 1, recieved 0"
  exit 1
fi
shift 1

# set default values for flags
LIST_FLAG=false
DRINK="beer"

while [[ $# -gt 0 ]]; do
  case $1 in
    -l|--list)
      LIST_FLAG=true
      shift
      ;;
  
    --drink)
      if [ $# -eq 1 ]; then
        echo "no value provided for --drink"
        exit 1
      fi

      DRINK=$2

      # check if value is any of: [water, beer, wine, coke]
      if [[ ! "$DRINK" =~ ^(water|beer|wine|coke)$ ]]; then
        echo "invalid value provided for drink: \"$DRINK\""
        exit 1
      fi

      shift 2
      ;;
  
    --help)
      echo -e "$__usage"
      exit
      ;;
    *)
      echo -e "Unkown flag \"$1\"\nTry foo --help for help."
      exit 1
      ;;
  esac
done

```

