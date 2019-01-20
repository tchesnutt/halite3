# Halite 3
Write up coming eventually.

## Notable Versions

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
