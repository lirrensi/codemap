interface Props {
  title: string;
  count: number;
}

type Handler = (event: Event) => void;

function Button({ title, count }: Props) {
  return `<button>${title}: ${count}</button>`;
}

const handleClick: Handler = (e) => {
  console.log(e);
};
