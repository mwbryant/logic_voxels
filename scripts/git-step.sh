#!/bin/bash
# For video history review
for commit in $(git rev-list master --reverse)
do
    git checkout $commit
    read
done