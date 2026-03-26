SLEEP=0

exec 8< <(cargo run r b.zy 2>/dev/null | head -n 50000)

while IFS="" read -r -u 8 line
do
  if [[ "${line}" == "" ]]
  then
    echo -en "\x1b[J"
    # echo -en '\x1b[K'
    if [[ ${SLEEP} == 0 ]]
    then
      read -n 1 -s
    else
      sleep ${SLEEP}
    fi
    echo -en "\x1b[1;1H"
  else
    echo -en "${line}"
    echo -e "\x1b[K"
  fi
done

# clear
