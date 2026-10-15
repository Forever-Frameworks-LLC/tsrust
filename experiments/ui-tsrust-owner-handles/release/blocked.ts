import {tracked, close} from './tracked.rs';
function main() {
  const value = tracked();
  close(value);
}
