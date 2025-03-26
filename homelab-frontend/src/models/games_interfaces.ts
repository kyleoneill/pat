export interface ConnectionGameRowInterface {
  category_name: string,
  category_clues: Array<string>,
}

export class ConnectionGameRow implements ConnectionGameRowInterface {
  category_name: string;
  category_clues: Array<string>;

  constructor() {
    this.category_name = '';
    this.category_clues = ['', '', '', ''];
  }
}

export class CreateConnectionsGame {
  puzzle_name: string;
  connection_categories: Array<ConnectionGameRow>;

  constructor() {
    this.puzzle_name = '';
    this.connection_categories = [];
    for (let i = 0; i < 4; i++) {
      this.connection_categories.push(new ConnectionGameRow());
    }
  }

  is_set(): boolean {
    if (this.puzzle_name === '' || this.connection_categories.length !== 4) {
      return false;
    }
    for (const row of this.connection_categories) {
      if (row.category_name === '' || row.category_clues.length !== 4) {
        return false;
      }
      for (const clue of row.category_clues) {
        if (clue === '') {
          return false;
        }
      }
    }
    return true;
  }
}
