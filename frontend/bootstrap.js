import './styles/style.scss';
// import './node_modules/yew-styles/main.css';

import("./pkg").then(module => {
  module.run_app();
});
