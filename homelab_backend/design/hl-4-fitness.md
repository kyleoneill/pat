# Fitness
A fitness feature should be composed of two major components, nutrition and exercise.

## Nutrition
The nutrition component should require a user to create a "nutrition profile" where they input their height and weight
with their desired weight, with desired weekly changes in pounds (0, +/- 0.5, +/- 1). The backend will then calculate
daily calories required to meet this goal.

This component should allow a user to input meals, which track against daily calorie and macro-nutrient goals. Meals
should be either an individual food item or a bundling of individual food items that a user can save under a name
(e.g., a user creates some 'fruit smoothie' composed of a cup of yogurt, 3 cups of milk, 2 cups of frozen fruit,
ex).

### Nutrition Data
Individual food items will need to come from some external source. It may be possible to query the US government
for it at https://fdc.nal.usda.gov/ and then save responses to use local data on subsequent searches. This may prove
difficult to implement for a user searching for a specific thing (a search for "Milk" there are many responses,
how do I handle picking one?). This resource has JSON files that can be queried locally. The foundational food one
is only a few MB, but the branded one is 3GB. This download is more realistic, but I will need to figure out how
to store and read from it locally as a 3GB JSON file may be hard for a Pi to read through on each request.

Maybe I can save just a list of names of all food items and then a user can fuzzy search by name in this shortened
collection of data. We can then retrieve the full data for a food item after it's selected. Maybe the full data
can be stored in the DB, but I will need some startup job that fills the DB if the data is not present. The full
data could be stored in a list of files (maybe segmented alphabetically by first letter, one file per letter?),
but this will be less efficient for reading as each request will have to parse a file that is still likely to be large.

### New Structs
Each of these will need some identifier. The nutrition profile, food component, and meal structs should be associated
with a user.

- Nutrition Profile
  - Height feet
  - Height inches
  - Weight
  - Current weight
  - Desired weight
  - Weekly desired change
- Food Item
  - Calories
  - Protein
  - Fat
  - Other data
- Food Component
  - Food item
  - Unit of measure (like, 'cup')
  - Amount
    - This will just be a number and paired with "unit of measure"
- Meal
  - List of Food Component

## Exercise
A user should be able to enter a workout, which contains a date and a list of exercises done. This list should be a
bundling of an exercise type, sets, and reps. For convenience, a user should also be able to create a saved grouping
of exercises, sets, and reps so a bundling of data can be used repeatedly.

### New Structs
- Exercise
  - Name
  - Body part
- WorkoutExercise
  - `Exercise`
  - Sets
  - Reps
- ExerciseGroup
  - Name
  - List of `WorkoutExercise`
- Workout
  - User identifier
  - Date
  - List of `WorkoutExercise`
    - The frontend should allow a user to select an `ExerciseGroup` to autopopulate this
  - Workout length
