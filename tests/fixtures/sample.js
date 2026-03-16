class EventEmitter {
  constructor() {
    this.listeners = [];
  }

  on(event, handler) {
    this.listeners.push({ event, handler });
  }

  emit(event, data) {
    this.listeners
      .filter((l) => l.event === event)
      .forEach((l) => l.handler(data));
  }
}

function add(a, b) {
  return a + b;
}

const multiply = (a, b) => a * b;

export function greet(name) {
  return `Hello, ${name}!`;
}
