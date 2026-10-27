import React from 'react';
import ReactDOM from 'react-dom/client';
import Tasks from '../../web/src/Tasks';
import { session } from './lib/api';
import { TitleBar } from './components/title-bar';
import './renderer.css';

ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode><TitleBar /><Tasks initialSession={session} /></React.StrictMode>);
