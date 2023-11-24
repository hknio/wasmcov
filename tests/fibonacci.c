#include <stdio.h>
 
int main()
{
   int count = 30;
   int first = 0;
   int second = 0;
   int value;

   for (int i = 0; i < count; i++ )
   {
      if ( i <= 1 )
         value = i;
      else
      {
         value = first + second;
         first = second;
         second = value;
      }
      printf("%d\n", value);
   }
 
   return 0;
}