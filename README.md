# Halite 3
Halite, fun game. I wish I started sooner as I only had 10 days for submitting. If I did better, I would feel compelled to write up my implementation. In hindsight, I wish I had better target selection and path finding. My ships only looked on turn in advance when moving and that was to avoid collisions. I felt this limiting when I started my dropoff strategy, as in order to get a dropoff out early you need to plan at least 4 moves in advance. I had a great time this year though. This was my first project with Rust, it was a joy to code. 

## Notable Versions

*v14*
* Added drop-off logic


*v8*
* Refactored Navigation to look one move in advance. Ships who have a move that's more important go first.
* Anti cheese protections broke this version. Bot ends up crashing ~10% of the time.

*v6*
* Added blur function to computation of value
* Added cell distance from drop-off function to computation of value
* Fixed major bug with gather_move function

*v4*
* Added logic to complete go home 

*v3* 
* Refactor: move logic to Navi
* Enemy ship avoidance added unless near shipyard
* Basic inspiration tracking hooked up
* Better go home logic for end of game

*v2* 
* Most collisions avoided
* Basic _flawed_ move logic
