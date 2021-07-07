# pps - Primordial Particle System

This program aims to simulate the system in the research article "How a life-like 
system emerges from a simplistic particle motion law."

## Notes

To speed up the simulation, the particles are segregated into blocks with size 
equal to radius r. Hence, we do not need to iterate over all particles twice (n^2), 
but only need to iterate over all particles once then particles within blocks 
nearby (n*m).

## 🔰 Commit Emoji Guide

| Emoji          | Meaning        |
| -------------- | -------------- |
| :bug:          | Bugfix         |
| :package:      | Dependency     |
| :no_entry:     | Deprecation    |
| :book:         | Documentation  |
| :sparkles:     | Features       |
| :construction: | In-Progress    |
| :zap:          | Performance    |
| :recycle:      | Refactoring    |
| :lock:         | Security       |
| :test_tube:    | Tests          |
| :pencil:       | Typos          |
| :lipstick:     | UI / Cosmetic  |
| :bookmark:     | Version        |
|                |                |
| :tada:         | Initial Commit |
| :rocket:       | Release        |
| :rewind:       | Revert Changes |
