class Test2 {
    int myField;

    public static void main(String[] args) {
        int i = 0;
        i++;
        new Test2().print(i);
    }

    void print(int i) {
        System.out.println(i);
    }
}