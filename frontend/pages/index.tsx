import type { NextPage } from 'next';
import Head from 'next/head';
import styles from '../styles/Home.module.css';
import Link from 'next/link';
const Home: NextPage = () => {
  return (
    <div className={styles.container}>
      <Head>
        <title>欢迎使用T大地洞</title>
        <meta name='description' content='Generated by create next app' />
        <link rel='icon' href='/favicon.ico' />
      </Head>
      <main className={styles.main}>
        <h1 className={styles.title}>欢迎来到T大地洞</h1>

        <p className={styles.description}>
          <Link href='./login'>即刻登录 </Link>
        </p>

        <div className={styles.grid}></div>
      </main>

      <footer className={styles.footer}>
        <a>
          <span className={styles.logo}></span>
        </a>
      </footer>
    </div>
  );
};

export default Home;