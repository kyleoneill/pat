<script setup lang="ts">
  import type { Ref } from 'vue';
  import { ref } from 'vue';
  import { RouterLink, useRoute } from 'vue-router';

  import type { ScrambledGame } from "@/models/games_interfaces";

  import Toaster from "@/components/ToasterComponent.vue";
  import useToasterStore from '@/stores/useToasterStore'
  const toasterStore = useToasterStore();

  import { getConnectionGameToPlay, trySolveConnectionGameRow } from "@/api/games_api";

  const route = useRoute();

  const loading = ref(false);

  class GameState {
    clues: Array<string>;
    selectedCells: Array<number>;
    knownCategories: Array<string>;
    interactableIndex: number;

    constructor() {
      this.clues = [];
      this.selectedCells = [];
      this.knownCategories = ['?', '?', '?', '?'];
      this.interactableIndex = 0;
    }

    selectClue(index: number): void {
      if (index < this.interactableIndex) {
        // Do not let the user select something that is considered solved
        return
      }
      const maybeIndex = this.selectedCells.indexOf(index);
      if(maybeIndex !== -1) {
        // We already have this item selected, so un-select it
        this.selectedCells.splice(maybeIndex, 1);
      }
      else {
        // Select a new item, if we have not yet selected 4
        if(this.selectedCells.length !== 4) {
          this.selectedCells.push(index);
        }
      }
    }

    solveRow(categoryName: string): void {
      // Have to turn the indexes into values and then do an IndexOf in the splice, because the first
      // splice will make the remaining values in this.selectedCells incorrect
      const actualClues: Array<string> = [
        this.clues[this.selectedCells[0]],
        this.clues[this.selectedCells[1]],
        this.clues[this.selectedCells[2]],
        this.clues[this.selectedCells[3]],
      ];

      // Empty out selected cells
      this.selectedCells = [];

      // Remove the correct clues from the clue list
      this.clues.splice(this.clues.indexOf(actualClues[0]), 1);
      this.clues.splice(this.clues.indexOf(actualClues[1]), 1);
      this.clues.splice(this.clues.indexOf(actualClues[2]), 1);
      this.clues.splice(this.clues.indexOf(actualClues[3]), 1);

      // Add the removed clues back as an ordered row underneath the previously solved row (if there is one)
      this.clues.splice(this.interactableIndex, 0, actualClues[0]);
      this.clues.splice(this.interactableIndex + 1, 0, actualClues[1]);
      this.clues.splice(this.interactableIndex + 2, 0, actualClues[2]);
      this.clues.splice(this.interactableIndex + 3, 0, actualClues[3]);

      // TODO: Display this better
      this.knownCategories[this.knownCategories.indexOf("?")] = categoryName;

      // Increment the index indicating where the barrier is between clues in the solved or un-solved state
      this.interactableIndex += 4;
    }
  }

  const currentGame: Ref<ScrambledGame | undefined> = ref();
  const gameState: Ref<GameState> = ref(new GameState());
  const gameSlug: Ref<string> = ref(route.params.gameSlug as string);

  function submitRow() {
    if(gameState.value.selectedCells.length === 4) {
      loading.value = true;
      const data: Array<string> = [
        gameState.value.clues[gameState.value.selectedCells[0]] as string,
        gameState.value.clues[gameState.value.selectedCells[1]] as string,
        gameState.value.clues[gameState.value.selectedCells[2]] as string,
        gameState.value.clues[gameState.value.selectedCells[3]] as string,
      ];
      trySolveConnectionGameRow(gameSlug.value, data).then(response => {
        if(response.data['correct_guess']) {
          gameState.value.solveRow(response.data['row_name']);
          toasterStore.success({text: response.data['row_name']});
        }
        else {
          // TODO: Make this nicer
          toasterStore.warning({text: "Incorrect guess"});
        }
      }).catch(error => {
        // TODO
      }).finally(() => {
        loading.value = false;
      });
    }
  }

  function getGameToPlay() {
    loading.value = true;
    getConnectionGameToPlay(gameSlug.value).then(response => {
      currentGame.value = response.data;
      gameState.value.clues = response.data['scrambled_clues'];
    }).catch(error => {
      // TODO
    }).finally(() => {
      loading.value = false;
    });
  }

  getGameToPlay();

  // TODO: The :class binding for a cell has code smell, is there a better way to emulate this behavior?
</script>

<template>
  <Toaster />
  <div class="section-header">
    <RouterLink class="router-button" to="/games/connections">Back</RouterLink>
  </div>
  <div v-if="currentGame !== undefined">
    <h2>{{currentGame.puzzle_name}}</h2>
    <h4>Known Categories</h4>
    <div>
      <div
        v-for="(categoryName, index) in gameState.knownCategories"
        :key="index"
      >
        {{index + 1}}: {{categoryName}}
      </div>
    </div>
    <div class="puzzle-grid">
      <div
        v-for="(clue, index) in gameState.clues"
        :key="index"
        class="puzzle-grid-cell"
        :class="{'highlight-clue': gameState.selectedCells.includes(index), 'solved-cell': index < gameState.interactableIndex, 'solved-cell-second': index < gameState.interactableIndex && index > 3, 'solved-cell-third': index < gameState.interactableIndex && index > 7, 'solved-cell-fourth': index < gameState.interactableIndex && index > 11}"
        @click="gameState.selectClue(index)"
      >
        {{ clue }}
      </div>
    </div>
    <button :disabled="loading" @click="submitRow">Submit</button>
  </div>
</template>

<style scoped>
  .puzzle-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .puzzle-grid-cell {
    padding: 1rem;
    background-color: var(--vt-c-black);
    border: 1px solid #ccc;
    cursor: pointer;
    text-align: center;
  }

  .solved-cell {
    cursor: unset;
    background-color: mediumpurple;
    color: #181818;
  }

  .solved-cell-second {
    background-color: cornflowerblue;
    color: #181818;
  }

  .solved-cell-third {
    background-color: #fcfc64;
    color: #181818;
  }

  .solved-cell-fourth {
    background-color: lightgreen;
    color: #181818;
  }

  .highlight-clue {
    background-color: ivory;
    color: var(--vt-c-text-light-3);
  }

  button {
    margin-top: 2rem;
    font-size: large;
    padding: 0.5rem 1rem 0.5rem 1rem;
  }
</style>
