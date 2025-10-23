#!/bin/bash

echo "Building programs"

mkdir -p tmp

# Test unchecked-account
mv programs/account-info tmp/
mv programs/ignore-non-accounts tmp/
output=$(anchor keys sync && anchor build 2>&1 > /dev/null)
if ! [[ $output =~ "Struct field \"unchecked\" is unsafe" ]]; then
   echo "Error: expected /// CHECK error in programs/unchecked-account"
   exit 1
fi
mv tmp/account-info programs/
mv tmp/ignore-non-accounts programs/

# Test account-info
mv programs/unchecked-account tmp/
mv programs/ignore-non-accounts tmp/
output=$(anchor keys sync && anchor build 2>&1 > /dev/null)
if ! [[ $output =~ "Struct field \"unchecked\" is unsafe" ]]; then
   echo "Error: expected /// CHECK error in programs/account-info"
   exit 1
fi
mv tmp/unchecked-account programs/
mv tmp/ignore-non-accounts programs/

# Test ignore-non-accounts
mv programs/unchecked-account tmp/
mv programs/account-info tmp/
if ! anchor keys sync && anchor build ; then
   echo "Error: anchor build failed when it shouldn't have in programs/ignore-non-accounts"
   exit 1
fi
mv tmp/unchecked-account programs/
mv tmp/account-info programs/

rmdir tmp

echo "Success. As expected, all builds failed that were supposed to fail."
